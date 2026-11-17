; scroll.s: a landscape two screens wide. Left and Right slide the view;
; the bar at the top stays where it is.
; Used from Part 19. Build it with:
;     cd code/nesasm
;     cargo run -- ../roms/scroll.s ../roms/scroll.nes

tick      = $00         ; set to 1 by the interrupt when a new picture starts
buttons   = $01
scroll_lo = $02         ; the view's left edge: 0 to 255...
scroll_hi = $03         ; ...in screen 0 or screen 1

WALL = $01
BAR  = $02
DOT  = $03

.mirror vertical        ; two name tables side by side

.font 3

.tile WALL
    22222221
    21111111
    21111111
    21111111
    22212222
    11121111
    11121111
    11121111

.tile BAR
    11111111
    11111111
    11111111
    11111111
    11111111
    11111111
    11111111
    33333333

.tile DOT               ; sprite 0: one solid line at the very bottom
    ........
    ........
    ........
    ........
    ........
    ........
    ........
    33333333

reset:
    SEI
    CLD
    LDX #$FF
    TXS
    INX
    STX $2000
    STX $2001
    STX scroll_lo
    STX scroll_hi
wait1:
    BIT $2002
    BPL wait1
wait2:
    BIT $2002
    BPL wait2

    LDA #$FF            ; park all 64 sprites below the screen
hide:
    STA $0200,X
    INX
    BNE hide

    LDA #$3F            ; colours
    STA $2006
    LDA #$00
    STA $2006
colours:
    LDA palette,X
    STA $2007
    INX
    CPX #$20
    BNE colours

    LDA #$20            ; the fixed bar: its text on row 1...
    STA $2006
    LDA #$24
    STA $2006
    LDX #$00
bar_text:
    LDA help,X
    STA $2007
    INX
    CPX #24
    BNE bar_text
    LDA #$20            ; ...and a solid strip on row 3
    STA $2006
    LDA #$60
    STA $2006
    LDX #32
    LDA #BAR
strip:
    STA $2007
    DEX
    BNE strip

    LDA #$21            ; a sign on each screen
    STA $2006
    LDA #$44
    STA $2006
    LDX #$00
    JSR sign
    LDA #$25
    STA $2006
    LDA #$44
    STA $2006
    LDX #10
    JSR sign

    LDA #$23            ; the ground: rows 26 to 29 of screen 0...
    STA $2006
    LDA #$40
    STA $2006
    JSR ground
    LDA #$27            ; ...and of screen 1
    STA $2006
    LDA #$40
    STA $2006
    JSR ground

    LDA #%00000100      ; towers are drawn downwards
    STA $2000
    LDX #$00
tower:
    LDA towers,X        ; three bytes each: address high, address low, height
    STA $2006
    LDA towers+1,X
    STA $2006
    LDY towers+2,X
    LDA #WALL
brick:
    STA $2007
    DEY
    BNE brick
    INX
    INX
    INX
    CPX #18
    BNE tower
    LDA #$00
    STA $2000

    LDA #$23            ; the ground and towers use palette 1
    STA $2006
    LDA #$E8
    STA $2006
    JSR greens
    LDA #$27
    STA $2006
    LDA #$E8
    STA $2006
    JSR greens

    LDA #23             ; sprite 0 sits on the last line of the bar
    STA $0200
    LDA #DOT
    STA $0201
    LDA #$00
    STA $0202
    LDA #8
    STA $0203

    LDA #%10000000
    STA $2000
    LDA #%00011110
    STA $2001

main:
    LDA tick
    BEQ main            ; wait for the interrupt
    LDA #$00
    STA tick

    JSR read_pad
    LDA buttons
    AND #$80            ; Right: the view moves right
    BEQ no_right
    LDA scroll_lo
    CLC
    ADC #2
    STA scroll_lo
    BCC no_right
    LDA scroll_hi       ; past 255: over to the other screen
    EOR #$01
    STA scroll_hi
no_right:
    LDA buttons
    AND #$40            ; Left
    BEQ no_left
    LDA scroll_lo
    SEC
    SBC #2
    STA scroll_lo
    BCS no_left
    LDA scroll_hi
    EOR #$01
    STA scroll_hi
no_left:

; The top of the picture is being drawn unscrolled. Wait until the PPU
; reaches sprite 0, at the bottom of the bar, and only then set the scroll.
wait_clear:
    BIT $2002
    BVS wait_clear      ; last frame's "sprite 0" flag is still up
wait_hit:
    BIT $2002
    BVC wait_hit        ; not there yet
    LDA scroll_hi
    ORA #%10000000
    STA $2000           ; which screen...
    LDA scroll_lo
    STA $2005           ; ...and how far into it
    LDA #$00
    STA $2005
    JMP main

sign:                   ; ten letters starting at signs+X
    LDY #10
sign_loop:
    LDA signs,X
    STA $2007
    INX
    DEY
    BNE sign_loop
    RTS

ground:                 ; four rows of wall
    LDX #128
    LDA #WALL
ground_loop:
    STA $2007
    DEX
    BNE ground_loop
    RTS

greens:                 ; the last three bands of the colour map
    LDX #24
    LDA #$55
greens_loop:
    STA $2007
    DEX
    BNE greens_loop
    RTS

read_pad:
    LDA #$01
    STA $4016
    LDA #$00
    STA $4016
    LDX #8
read_bit:
    LDA $4016
    LSR A
    ROR buttons
    DEX
    BNE read_bit
    RTS

nmi:
    PHA
    LDA #$00            ; copy the sprite list at $0200 to the PPU
    STA $2003
    LDA #$02
    STA $4014
    LDA #%10000000      ; the bar at the top is never scrolled
    STA $2000
    LDA #$00
    STA $2005
    STA $2005
    LDA #$01
    STA tick
    PLA
    RTI

irq:
    RTI

help:
    .byte "LEFT AND RIGHT TO SCROLL"
signs:
    .byte "SCREEN ONE"
    .byte "SCREEN TWO"

; address high, address low, height: each tower ends on the ground at row 26
towers:
    .byte $22, $E6, 3
    .byte $22, $8E, 6
    .byte $23, $18, 2
    .byte $26, $A4, 5
    .byte $26, $90, 6
    .byte $26, $DB, 4

palette:
    .byte $21, $00, $10, $30    ; background 0: sky blue, greys, white
    .byte $21, $09, $19, $29    ; background 1: greens
    .byte $21, $00, $10, $30
    .byte $21, $00, $10, $30
    .byte $21, $00, $10, $30    ; sprites
    .byte $21, $00, $10, $30
    .byte $21, $00, $10, $30
    .byte $21, $00, $10, $30

.vectors nmi, reset, irq
