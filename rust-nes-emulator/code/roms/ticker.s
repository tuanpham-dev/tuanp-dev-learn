; ticker.s: a heart walks across the screen, one step every eight frames.
; Used from Part 15. Build it with:
;     cd code/nesasm
;     cargo run -- ../roms/ticker.s ../roms/ticker.nes

frame  = $00            ; counts pictures
column = $01            ; where the heart is, 0 to 31

.font 3

.tile 1                 ; the heart from hello.nes
    .33.33..
    3223223.
    3122223.
    3222223.
    .32223..
    ..323...
    ...3....
    ........

reset:
    SEI                 ; the usual start-up routine:
    CLD
    LDX #$FF
    TXS                 ; the stack starts at the top of its page
    INX                 ; X is now 0
    STX $2000           ; no interrupts from the PPU yet
    STX $2001           ; picture off
wait1:
    BIT $2002           ; wait for the PPU to finish a picture...
    BPL wait1
wait2:
    BIT $2002           ; ...twice, so that it has warmed up
    BPL wait2

    LDA #$3F            ; the colours go to PPU address $3F00
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

    LDA #$20            ; row 4, column 10
    STA $2006
    LDA #$8A
    STA $2006
    LDX #title-strings
    JSR print

    LDA #$21            ; row 8, column 6
    STA $2006
    LDA #$06
    STA $2006
    LDX #line2-strings
    JSR print

    LDA #$22            ; row 22, column 7
    STA $2006
    LDA #$C7
    STA $2006
    LDX #line3-strings
    JSR print

    LDA #$23            ; which palette each part of the screen uses:
    STA $2006           ; PPU address $23C0, eight bytes per band
    LDA #$C0
    STA $2006
    LDY #$00
band:
    LDA bands,Y
    LDX #$08
band_fill:
    STA $2007
    DEX
    BNE band_fill
    INY
    CPY #$08
    BNE band

    LDA #$00
    STA frame
    STA column
    STA $2005           ; no scrolling
    STA $2005
    LDA #%10000000      ; ask for an interrupt at the end of every picture
    STA $2000
    LDA #%00001010      ; picture on
    STA $2001
forever:
    JMP forever         ; everything else happens in the interrupt

; Send the text that starts at strings+X to the PPU, up to a zero byte.
print:
    LDA strings,X
    BEQ print_done
    STA $2007
    INX
    BNE print
print_done:
    RTS

; Sixty times a second the PPU interrupts us here.
nmi:
    PHA
    INC frame
    LDA frame
    AND #$07            ; only every eighth frame...
    BNE nmi_done

    LDA #$21            ; rub out the heart: row 14 starts at $21C0
    STA $2006
    LDA column
    CLC
    ADC #$C0
    STA $2006
    LDA #$00
    STA $2007

    INC column          ; one step to the right, wrapping at 32
    LDA column
    AND #$1F
    STA column

    LDA #$21            ; draw it again
    STA $2006
    LDA column
    CLC
    ADC #$C0
    STA $2006
    LDA #$01
    STA $2007
nmi_done:
    LDA #%10000000      ; put the PPU back where a picture starts
    STA $2000
    LDA #$00
    STA $2005
    STA $2005
    PLA
    RTI

irq:
    RTI

strings:
title:
    .byte "HELLO AGAIN", 0
line2:
    .byte "ONE PICTURE AT A TIME", 0
line3:
    .byte "SIXTY EVERY SECOND", 0

; one palette number (0 to 3, repeated four times in a byte) per band of 32 lines
bands:
    .byte $00, $00, $AA, $55, $00, $FF, $00, $00

palette:
    .byte $0C, $00, $10, $30    ; background 0: white on dark blue-green
    .byte $0C, $30, $16, $06    ; background 1: reds, for the heart
    .byte $0C, $0A, $1A, $2A    ; background 2: greens
    .byte $0C, $08, $18, $28    ; background 3: yellows
    .byte $0C, $30, $27, $16    ; sprites (not used here)
    .byte $0C, $30, $27, $16
    .byte $0C, $30, $27, $16
    .byte $0C, $30, $27, $16

.vectors nmi, reset, irq
