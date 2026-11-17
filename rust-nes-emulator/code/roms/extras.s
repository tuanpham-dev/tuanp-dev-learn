; extras.s: four things real cartridges do that our earlier test programs
; did not. Used in Part 20. Build it with:
;     cd code/nesasm
;     cargo run -- ../roms/extras.s ../roms/extras.nes
;
;  1. the background uses the SECOND set of 256 tiles
;  2. sprites are 8 pixels wide and 16 high
;  3. the left 8 pixels of the picture are hidden
;  4. the program reads a byte back from the PPU's memory

tick = $00

.font 3, 1              ; the font goes in the second tile set

.tile $104              ; a stripe, also in the second set
    11111111
    22222222
    33333333
    22222222
    11111111
    22222222
    33333333
    22222222

.tile $10               ; a tall figure: sprite tiles $10 (top) and $11
    ..3333..
    .333333.
    .313313.
    .333333.
    ..3223..
    ...33...
    .222222.
    22222222

.tile $11
    2.2222.2
    2.2222.2
    ..2222..
    ..1111..
    ..1..1..
    ..1..1..
    ..1..1..
    .33..33.

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

    LDA #$20            ; a stripe right across row 6, from the very left edge
    STA $2006
    LDA #$C0
    STA $2006
    LDX #32
    LDA #$04
stripe:
    STA $2007
    DEX
    BNE stripe

    LDA #$21            ; "TALL SPRITES" at row 10, column 4
    STA $2006
    LDA #$44
    STA $2006
    LDX #$00
text1:
    LDA words,X
    STA $2007
    INX
    CPX #12
    BNE text1

    LDA #$21            ; "READ BACK: " at row 14, column 4
    STA $2006
    LDA #$C4
    STA $2006
    LDX #$00
text2:
    LDA words2,X
    STA $2007
    INX
    CPX #11
    BNE text2

    LDA #$21            ; read the first letter of "TALL SPRITES" back...
    STA $2006
    LDA #$44
    STA $2006
    LDA $2007           ; (the first read only fills the PPU's buffer)
    LDA $2007           ; this is the real one: "T"
    TAX
    LDA #$21            ; ...and write it after "READ BACK: "
    STA $2006
    LDA #$CF
    STA $2006
    STX $2007

    LDX #$00            ; three tall sprites; the last one hangs over the left edge
sprites:
    LDA figures,X
    STA $0200,X
    INX
    CPX #12
    BNE sprites

    LDA #$00
    STA $2005
    STA $2005
    LDA #%10110000      ; interrupts on, tall sprites, background from tile set 2
    STA $2000
    LDA #%00011000      ; background and sprites on, left 8 pixels hidden
    STA $2001
forever:
    JMP forever

nmi:
    PHA
    LDA #$00
    STA $2003
    LDA #$02
    STA $4014
    PLA
    RTI

irq:
    RTI

words:
    .byte "TALL SPRITES"
words2:
    .byte "READ BACK: "

; y, tile, attributes, x
figures:
    .byte 100, $10, $00, 160
    .byte 100, $10, $41, 190        ; mirrored, other palette
    .byte 100, $10, $00, 4          ; half hidden by the left edge

palette:
    .byte $0F, $11, $21, $30    ; background: blues and white on black
    .byte $0F, $11, $21, $30
    .byte $0F, $11, $21, $30
    .byte $0F, $11, $21, $30
    .byte $0F, $27, $16, $30    ; sprite palette 0
    .byte $0F, $2A, $12, $30    ; sprite palette 1
    .byte $0F, $27, $16, $30
    .byte $0F, $27, $16, $30

.vectors nmi, reset, irq
