; paddle.s: a small bat-and-ball game written for this course.
; Left and Right move the bat, Start serves the ball.
; Every bounce off the bat is a point; every ten points the ball speeds up.
; Build it with:
;     cd code/nesasm
;     cargo run -- ../roms/paddle.s ../roms/paddle.nes

tick     = $00          ; set to 1 by the interrupt when a new picture starts
buttons  = $01          ; the controller: A B Select Start Up Down Left Right
ball_x   = $02
ball_y   = $03
ball_dx  = $04          ; how far the ball moves each frame; $FF is -1, $FE is -2
ball_dy  = $05
paddle_x = $06
playing  = $07          ; 0 while waiting for Start
score0   = $08          ; three decimal digits: ones, tens, hundreds
score1   = $09
score2   = $0A
dirty    = $0B          ; 1 when the score on screen needs redrawing
message  = $0C          ; 1: show PRESS START, 2: rub it out

BALL     = $01          ; tile numbers
PAD_END  = $02
PAD_MID  = $03
WALL     = $04

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

.tile PAD_END
    ..111111
    .1222222
    12333333
    12333333
    12333333
    12333333
    .1222222
    ..111111

.tile PAD_MID
    11111111
    22222222
    33333333
    33333333
    33333333
    33333333
    22222222
    11111111

.tile WALL
    22222221
    21111111
    21111111
    21111111
    22212222
    11121111
    11121111
    11121111

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

    LDA #$00            ; clear our variables...
    TAX
clear:
    STA $00,X
    INX
    BNE clear
    LDA #$FF            ; ...and park all 64 sprites below the screen
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

    LDA #$20            ; the top wall: row 2, all 32 columns
    STA $2006
    LDA #$40
    STA $2006
    LDX #32
    LDA #WALL
top:
    STA $2007
    DEX
    BNE top

    LDA #%00000100      ; make the PPU step DOWN a row after each write
    STA $2000
    LDA #$20            ; the left wall: column 0, from row 3 down
    STA $2006
    LDA #$60
    STA $2006
    LDX #27
    LDA #WALL
left:
    STA $2007
    DEX
    BNE left
    LDA #$20            ; the right wall: column 31
    STA $2006
    LDA #$7F
    STA $2006
    LDX #27
    LDA #WALL
right:
    STA $2007
    DEX
    BNE right
    LDA #$00            ; back to stepping across
    STA $2000

    LDA #$20            ; "SCORE 000" at row 1, column 2
    STA $2006
    LDA #$22
    STA $2006
    LDX #$00
label:
    LDA score_text,X
    STA $2007
    INX
    CPX #9
    BNE label

    LDA #1              ; ask the interrupt to show PRESS START
    STA message
    LDA #104
    STA paddle_x
    JSR new_ball
    JSR update_sprites

    LDA #%10000000      ; interrupt at the end of every picture
    STA $2000
    LDA #%00011110      ; background and sprites on
    STA $2001

; The main loop runs once per picture.
main:
    LDA tick
    BEQ main            ; wait for the interrupt
    LDA #$00
    STA tick

    JSR read_pad
    JSR move_paddle

    LDA playing
    BNE play
    LDA buttons         ; not playing: is Start down?
    AND #$08
    BEQ draw
    LDA #1
    STA playing
    STA dirty
    LDA #2
    STA message
    LDA #$00
    STA score0
    STA score1
    STA score2
    JMP draw
play:
    JSR move_ball
draw:
    JSR update_sprites
    JMP main

; Read the eight buttons into `buttons`, one bit at a time.
read_pad:
    LDA #$01
    STA $4016           ; "take a snapshot of the buttons"
    LDA #$00
    STA $4016
    LDX #8
read_bit:
    LDA $4016           ; the next button arrives in bit 0
    LSR A               ; move it into the carry flag...
    ROR buttons         ; ...and from there into `buttons`
    DEX
    BNE read_bit
    RTS

move_paddle:
    LDA buttons
    AND #$40            ; Left
    BEQ no_left
    LDA paddle_x
    CMP #10
    BCC no_left         ; already at the wall
    SEC
    SBC #3
    STA paddle_x
no_left:
    LDA buttons
    AND #$80            ; Right
    BEQ no_right
    LDA paddle_x
    CMP #222
    BCS no_right
    CLC
    ADC #3
    STA paddle_x
no_right:
    RTS

new_ball:
    LDA #124
    STA ball_x
    LDA #100
    STA ball_y
    LDA #1
    STA ball_dx
    LDA #2
    STA ball_dy
    RTS

move_ball:
    LDA ball_x          ; sideways
    CLC
    ADC ball_dx
    STA ball_x
    CMP #8
    BCS not_left
    LDA #8              ; hit the left wall
    STA ball_x
    JSR turn_x
not_left:
    LDA ball_x
    CMP #240
    BCC not_right
    LDA #240            ; hit the right wall
    STA ball_x
    JSR turn_x
not_right:
    LDA ball_y          ; up or down
    CLC
    ADC ball_dy
    STA ball_y
    CMP #24
    BCS not_top
    LDA #24             ; hit the top wall
    STA ball_y
    JSR turn_y
not_top:
    LDA ball_dy         ; the bat only matters on the way down
    BMI ball_done
    LDA ball_y
    CMP #200
    BCC ball_done       ; still above the bat
    CMP #209
    BCS below           ; already past it

    LDA ball_x          ; is the ball over the bat?
    CLC
    ADC #7
    CMP paddle_x
    BCC ball_done       ; no: it is to the left
    LDA paddle_x
    CLC
    ADC #24
    CMP ball_x
    BCC ball_done       ; no: it is to the right

    LDA #200            ; yes: bounce
    STA ball_y
    JSR turn_y
    LDA ball_x          ; where on the bat did it land?
    CLC
    ADC #4
    SEC
    SBC paddle_x
    BMI hit_left
    CMP #8
    BCC hit_left
    CMP #16
    BCS hit_right
    JMP add_point       ; the middle: keep going the same way
hit_left:
    LDA #$FE            ; the left end sends it left
    STA ball_dx
    JMP add_point
hit_right:
    LDA #$02            ; the right end sends it right
    STA ball_dx
    JMP add_point

below:
    LDA ball_y
    CMP #232
    BCC ball_done
    LDA #$00            ; missed: wait for Start again
    STA playing
    LDA #1
    STA message
    JSR new_ball
ball_done:
    RTS

turn_x:
    LDA #$00            ; ball_dx = 0 - ball_dx
    SEC
    SBC ball_dx
    STA ball_dx
    RTS

turn_y:
    LDA #$00
    SEC
    SBC ball_dy
    STA ball_dy
    RTS

add_point:
    LDA #1
    STA dirty
    INC score0
    LDA score0
    CMP #10
    BNE point_done
    LDA #$00            ; carry into the tens
    STA score0
    LDA ball_dy         ; and every ten points, a faster ball
    CMP #$FC
    BEQ no_faster
    DEC ball_dy
no_faster:
    INC score1
    LDA score1
    CMP #10
    BNE point_done
    LDA #$00            ; carry into the hundreds
    STA score1
    INC score2
    LDA score2
    CMP #10
    BNE point_done
    LDA #$00
    STA score2
point_done:
    RTS

; Write the positions of the ball and the bat into the sprite list at $0200.
; Each sprite is four bytes: y, tile, attributes, x.
update_sprites:
    LDX ball_y          ; sprite 0: the ball
    DEX                 ; (sprites appear one line lower than their y)
    STX $0200
    LDA #BALL
    STA $0201
    LDA #$00
    STA $0202
    LDA ball_x
    STA $0203

    LDA #207            ; sprites 1 to 3: the bat
    STA $0204
    STA $0208
    STA $020C
    LDA #PAD_END
    STA $0205
    STA $020D
    LDA #PAD_MID
    STA $0209
    LDA #$01            ; palette 1
    STA $0206
    STA $020A
    LDA #$41            ; palette 1, mirrored left to right
    STA $020E
    LDA paddle_x
    STA $0207
    CLC
    ADC #8
    STA $020B
    CLC
    ADC #8
    STA $020F
    RTS

; Sixty times a second: the only safe moment to talk to the PPU.
nmi:
    PHA
    TXA
    PHA
    TYA
    PHA

    LDA #$00            ; copy the sprite list at $0200 to the PPU
    STA $2003
    LDA #$02
    STA $4014

    LDA dirty
    BEQ no_score
    LDA #$20            ; the three digits: row 1, column 8
    STA $2006
    LDA #$28
    STA $2006
    LDA score2
    ORA #$30            ; digit to character: 0 is tile $30, 1 is $31...
    STA $2007
    LDA score1
    ORA #$30
    STA $2007
    LDA score0
    ORA #$30
    STA $2007
    LDA #$00
    STA dirty
no_score:

    LDA message
    BEQ no_message
    LDA #$21            ; row 14, column 10
    STA $2006
    LDA #$CA
    STA $2006
    LDX #$00
message_loop:
    LDA #$20            ; a space...
    LDY message
    CPY #2
    BEQ put
    LDA start_text,X    ; ...or the next letter
put:
    STA $2007
    INX
    CPX #11
    BNE message_loop
    LDA #$00
    STA message
no_message:

    LDA #%10000000      ; put the PPU back where a picture starts
    STA $2000
    LDA #$00
    STA $2005
    STA $2005

    LDA #1
    STA tick
    PLA
    TAY
    PLA
    TAX
    PLA
    RTI

irq:
    RTI

score_text:
    .byte "SCORE 000"
start_text:
    .byte "PRESS START"

palette:
    .byte $0F, $00, $10, $30    ; background: greys and white on black
    .byte $0F, $00, $10, $30
    .byte $0F, $00, $10, $30
    .byte $0F, $00, $10, $30
    .byte $0F, $18, $28, $38    ; sprite palette 0: the ball, yellow
    .byte $0F, $01, $11, $21    ; sprite palette 1: the bat, blue
    .byte $0F, $00, $10, $30
    .byte $0F, $00, $10, $30

.vectors nmi, reset, irq
