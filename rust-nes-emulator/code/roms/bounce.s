; bounce.s: two balls bounce around the screen. One passes in front of the
; text and one behind it. Four arrows show the same tile flipped four ways.
; Used from Part 17. Build it with:
;     cd code/nesasm
;     cargo run -- ../roms/bounce.s ../roms/bounce.nes

tick = $00              ; set to 1 by the interrupt when a new picture starts
bx   = $10              ; two balls: x positions in $10 and $11,
by   = $12              ; y positions,
bdx  = $14              ; and how far each moves per frame ($FF is -1)
bdy  = $16

BALL  = $01
ARROW = $02

.font 3

.tile BALL
    ..1111..
    .122221.
    12233221
    12333321
    12333321
    12233221
    .122221.
    ..1111..

.tile ARROW
    3333333.
    333333..
    33333...
    333333..
    3333333.
    33.33333
    3...333.
    .....3..

reset:
    SEI
    CLD
    LDX #$FF
    TXS
    INX
    STX $2000
    STX $2001
wait1:
    BIT $2002
    BPL wait1
wait2:
    BIT $2002
    BPL wait2

    LDA #$FF            ; park all 64 sprites below the screen
    LDX #$00
hide:
    STA $0200,X
    INX
    BNE hide

    LDA #$3F            ; colours
    STA $2006
    LDA #$00
    STA $2006
    LDX #$00
colours:
    LDA palette,X
    STA $2007
    INX
    CPX #$20
    BNE colours

    LDA #$21            ; the text: row 14, column 6
    STA $2006
    LDA #$C6
    STA $2006
    LDX #$00
text:
    LDA words,X
    STA $2007
    INX
    CPX #19
    BNE text

    LDX #$00            ; the four arrows never move: copy them once
arrows:
    LDA arrow_sprites,X
    STA $0208,X
    INX
    CPX #16
    BNE arrows

    LDA #40             ; where the two balls start, and which way they go
    STA bx
    LDA #200
    STA bx+1
    LDA #60
    STA by
    LDA #150
    STA by+1
    LDA #1
    STA bdx
    STA bdy
    LDA #$FE
    STA bdx+1
    LDA #$FF
    STA bdy+1

    LDA #%10000000      ; interrupt at the end of every picture
    STA $2000
    LDA #%00011110      ; background and sprites on
    STA $2001

main:
    LDA tick
    BEQ main            ; wait for the interrupt
    LDA #$00
    STA tick

    LDX #$01            ; move ball 1, then ball 0
move:
    LDA bx,X
    CLC
    ADC bdx,X
    STA bx,X
    CMP #248            ; past the right edge (or wrapped past the left)?
    BCC x_ok
    LDA bx,X            ; step back, and turn round
    SEC
    SBC bdx,X
    STA bx,X
    LDA #$00
    SEC
    SBC bdx,X
    STA bdx,X
x_ok:
    LDA by,X
    CLC
    ADC bdy,X
    STA by,X
    CMP #224
    BCC y_ok
    LDA by,X
    SEC
    SBC bdy,X
    STA by,X
    LDA #$00
    SEC
    SBC bdy,X
    STA bdy,X
y_ok:
    DEX
    BPL move

    LDA by              ; sprite 0: ball 0, in front of the background
    STA $0200
    LDA #BALL
    STA $0201
    LDA #$00
    STA $0202
    LDA bx
    STA $0203

    LDA by+1            ; sprite 1: ball 1, palette 1, BEHIND the background
    STA $0204
    LDA #BALL
    STA $0205
    LDA #$21
    STA $0206
    LDA bx+1
    STA $0207
    JMP main

nmi:
    PHA
    LDA #$00            ; copy the sprite list at $0200 to the PPU
    STA $2003
    LDA #$02
    STA $4014
    LDA #$01
    STA tick
    PLA
    RTI

irq:
    RTI

words:
    .byte "IN FRONT AND BEHIND"

; four sprites: y, tile, attributes, x
arrow_sprites:
    .byte 40, ARROW, $02, 100       ; as drawn
    .byte 40, ARROW, $42, 116       ; mirrored left to right
    .byte 40, ARROW, $82, 132       ; upside down
    .byte 40, ARROW, $C2, 148       ; both

palette:
    .byte $0F, $00, $10, $30    ; background: white on black
    .byte $0F, $00, $10, $30
    .byte $0F, $00, $10, $30
    .byte $0F, $00, $10, $30
    .byte $0F, $18, $28, $38    ; sprite palette 0: yellow
    .byte $0F, $06, $16, $26    ; sprite palette 1: red
    .byte $0F, $0A, $1A, $2A    ; sprite palette 2: green
    .byte $0F, $01, $11, $21    ; sprite palette 3: blue

.vectors nmi, reset, irq
