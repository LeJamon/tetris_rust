extern crate piston_window;
extern crate rand; 

use piston_window::*;

use piston_window::ButtonArgs;
use rand::seq::SliceRandom;
use rand::thread_rng;

use std::io::BufReader;
use std::fs::File;
use std::sync::Arc;
use std::sync::mpsc::TryRecvError;

#[derive(PartialEq, Copy, Clone)]
enum Type_tetrimino {
    I,
    O,
    T,
    L,
    J,
    Z,
    S
}

#[derive(Clone, Copy)]
struct Tetrimino{
    typ: Type_tetrimino, 
    couleur: [f32;4],
    forme:[[u8;4];4]
}


impl Tetrimino {
    const fn new(typ: Type_tetrimino)-> Self{
        match typ {
            Type_tetrimino::I => Tetrimino { typ: Type_tetrimino::I,
                couleur: [ 43.0 ,250.0 ,250.0,1.0 ],    // cyan
                forme: [[0, 0, 1, 0],
                        [0, 0, 1, 0],
                        [0, 0, 1, 0],
                        [0, 0, 1, 0]] },

            Type_tetrimino::O => Tetrimino { typ: Type_tetrimino::O,
                couleur: [ 239.0,216.0,7.0,1.0],    // jaune
                forme: [[0, 0, 0, 0],
                        [0, 0, 0, 0],
                        [0, 1, 1, 0],
                        [0, 1, 1, 0]] },

            Type_tetrimino::T => Tetrimino { typ: Type_tetrimino::T,
                couleur: [ 255.0,0.0,255.0,1.0],    // magenta
                forme: [[0, 1, 0, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] },
            Type_tetrimino::L => Tetrimino { typ: Type_tetrimino::L,
                couleur: [255.0,127.0,0.0,1.0],    // orange
                forme: [[0, 0, 1, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] },
            Type_tetrimino::J => Tetrimino { typ: Type_tetrimino::J,
                couleur: [ 3.0,34.0,76.0,1.0],    // bleu
                forme: [[1, 0, 0, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] },
            Type_tetrimino::Z => Tetrimino { typ: Type_tetrimino::Z,
                couleur: [ 255.0,0.0,0.0,1.0],    // rouge
                forme: [[1,1, 0, 0],
                        [0, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] },

            Type_tetrimino::S => Tetrimino { typ: Type_tetrimino::S,
                couleur: [ 52.0,201.0,36.0,1.0],    // vert
                forme: [[0, 1, 1, 0],
                        [1, 1, 0, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0]] },
                    
                }
        }
}

type Terrain = [[u8; 10];24];

struct Etat_jeu{
    game_over: bool,
    counteur_chute:u32,
    terrain: Terrain,
    sac:Vec<Tetrimino>,
    tetrimino_actuel:Tetrimino,
    prochain_tetrimino:Tetrimino,
    colone:i32,
    ligne:i32,
    key_map:[bool;6] // droite, gauche, rotation droite, rotation gauche, chute, chute rapide
}

fn action_touche(key_map :&mut [bool;6], btn_info:ButtonArgs){

    match btn_info.button{
        Button::Keyboard(Key::Left) => key_map[0]= true, //gauche
        Button::Keyboard(Key::Right) => key_map[1] = true,//droite
        Button::Keyboard(Key::Up) => key_map[2]= true,//rotation 1
        Button::Keyboard(Key::F) => key_map[3]= true, // rotation 2
        Button::Keyboard(Key::Down) => key_map[4] = true, // chute
        Button::Keyboard(Key::Space) => key_map[5]= true, // chute rapide
        _ => ()
    }
}


fn mise_a_jour(etat_jeu:&mut Etat_jeu){

    if etat_jeu.counteur_chute < 15{
        etat_jeu.counteur_chute+=1;
    }
    else{
        etat_jeu.counteur_chute=0;

        if collision(&etat_jeu.terrain, &etat_jeu.tetrimino_actuel, &(etat_jeu.ligne+1), &etat_jeu.colone)
        {
            fusionner_terrain(&mut etat_jeu.terrain, &etat_jeu.tetrimino_actuel, &etat_jeu.ligne, &etat_jeu.colone);
            etat_jeu.terrain= nettoyer_ligne(etat_jeu.terrain);

            if etat_jeu.sac.is_empty() {etat_jeu.sac = creer_sac();}
            etat_jeu.tetrimino_actuel=etat_jeu.prochain_tetrimino;
            etat_jeu.prochain_tetrimino = etat_jeu.sac.pop().unwrap();

            etat_jeu.ligne=2;
            etat_jeu.colone=3;

            if collision(&etat_jeu.terrain, &etat_jeu.tetrimino_actuel, &etat_jeu.ligne,&etat_jeu.colone){
                etat_jeu.game_over=true;
            }
        
        }
        else {etat_jeu.ligne+=1;} // descend la piece actuelle d'une ligne 

    }

    //gauche
    if etat_jeu.key_map[0] && !collision(&etat_jeu.terrain,&etat_jeu.tetrimino_actuel,&etat_jeu.ligne, &(etat_jeu.colone -1)){
        {etat_jeu.colone-=1;}
    }

    //droite
    if etat_jeu.key_map[1] && !collision(&etat_jeu.terrain,&etat_jeu.tetrimino_actuel,&etat_jeu.ligne, &(etat_jeu.colone +1)){
        {etat_jeu.colone+=1;}
    }

    //chute
    if etat_jeu.key_map[4] && !collision(&etat_jeu.terrain,&etat_jeu.tetrimino_actuel,&(etat_jeu.ligne+1), &etat_jeu.colone){
        {etat_jeu.ligne+=1;}
    }

    //chute rapide
    if etat_jeu.key_map[5] && !collision(&etat_jeu.terrain,&etat_jeu.tetrimino_actuel,&(etat_jeu.ligne+1), &etat_jeu.colone){
        for ligne in etat_jeu.ligne..24{
            if collision(&etat_jeu.terrain,&etat_jeu.tetrimino_actuel,&etat_jeu.ligne,&etat_jeu.colone){
                etat_jeu.ligne = ligne-1;
                break;
            }
        }
    }

    // rotation
    if etat_jeu.key_map[2]{
        rotation(&mut etat_jeu.tetrimino_actuel, false);
        if collision(&etat_jeu.terrain, &etat_jeu.tetrimino_actuel, &etat_jeu.ligne, &etat_jeu.colone){
            rotation(&mut etat_jeu.tetrimino_actuel,true);
        }
    }

    if etat_jeu.key_map[3]{
        rotation(&mut etat_jeu.tetrimino_actuel, true);
        if collision(&etat_jeu.terrain, &etat_jeu.tetrimino_actuel, &etat_jeu.ligne, &etat_jeu.colone){
            rotation(&mut etat_jeu.tetrimino_actuel,false);
        }
    }

    etat_jeu.key_map = [false;6]; // suppression des touches enregistrées

}

fn creer_sac()-> Vec<Tetrimino>{
        let mut sac : Vec<Tetrimino> = vec![Tetrimino::new(Type_tetrimino::I),
        Tetrimino::new(Type_tetrimino::O),
        Tetrimino::new(Type_tetrimino::T),
        Tetrimino::new(Type_tetrimino::L),
        Tetrimino::new(Type_tetrimino::J),
        Tetrimino::new(Type_tetrimino::Z),
        Tetrimino::new(Type_tetrimino::S)];

        sac.shuffle(&mut thread_rng());
        sac.shuffle(&mut thread_rng());
        sac.shuffle(&mut thread_rng());
        return sac;
}

fn collision(terrain:&Terrain, tetrimino:&Tetrimino, ligne:&i32, colone:&i32)-> bool{
    let mut ligne_terrain:i32;
    let mut colonne_terrain:i32;

    for ligne_tetrimino in 0..4{
        for colone_tetrimino in 0..4{
            if tetrimino.forme[ligne_tetrimino][colone_tetrimino]==0{continue;}

            ligne_terrain = ligne_tetrimino as i32 +*ligne;
            colonne_terrain = colone_tetrimino as i32 +*colone;

            if colonne_terrain <0 {return true;} 
            if colonne_terrain >9{return true;}
            if ligne_terrain > 23{return true;}

            if terrain[ligne_terrain as usize][colonne_terrain as usize] !=0 {return true;}
        }
    }
    return false
}

fn fusionner_terrain(terrain:&mut Terrain, tetrimino:&Tetrimino, ligne_terrain:&i32, colonne_terrain:&i32 ){
    for ligne in 0..4{
        for colone in 0..4{
            if tetrimino.forme[ligne][colone] == 0 {continue;}
            terrain[(*ligne_terrain + ligne as i32) as usize][(*colonne_terrain+colone as i32)as usize] = tetrimino.forme[ligne][colone];
        }
    }
}

fn nettoyer_ligne(terrain: Terrain)-> Terrain{
    let mut new_terrain = [[0;10];24];
    let mut new_terrain_ligne : usize = 23;

    for ancienne_ligne in (0..24).rev(){

        let mut pop_count = 0;
        for colone in 0..10{
            if terrain[ancienne_ligne][colone]!=0{pop_count+=1;}
        }

        if pop_count == 0 || pop_count==10 {continue; }

        if terrain[ancienne_ligne].iter().sum::<u8>()>0{
            new_terrain[new_terrain_ligne]=terrain[ancienne_ligne];
            new_terrain_ligne-=1;
        }
    }
    return new_terrain;
}

fn rotation(tetrimino:&mut Tetrimino, clockwise:bool){

    if tetrimino.typ == Type_tetrimino::O{return;} // inutile de faire la rotation sur le carré 

    let source = tetrimino.forme;
    let mut rot :[[u8;4];4] = [[0;4];4];

    let taille:usize;
    if tetrimino.typ == Type_tetrimino::I{taille=4;}else{taille=3;}

    for ligne in 0..taille{

        if clockwise{
            for colone in 0..taille{
                rot[colone][(taille -1) - ligne]=source[ligne][colone];
            }
        }

        else {
            for colone in 0..taille{
                rot[(taille -1)-colone][ligne]=source[ligne][colone];
            }
        }
    }

    tetrimino.forme=rot;

}

fn rendu(win : &mut PistonWindow, re :&Event, ligne:&i32, colone:&i32, actuel:&Tetrimino, prochain: &Tetrimino, terrain: &Terrain){
    win.draw_2d(re, |_context, graphics, _device| {clear([0.5;4], graphics); });

    win.draw_2d(re, |context, graphics, _device| { rectangle([0.0, 0.0, 0.0, 1.0], [463.0, -140.0, 354.0, 842.0], context.transform, graphics); } );

    draw_terrain(win, re, terrain);                      // Draw the contents of the playfield.
    draw_tetrimino(win, re, ligne, colone, actuel);         // Draw the currently falling tetrimino.
    draw_prochain_tetrimino(win, re, 320.0, 115.0, prochain);    // Draw the next tetrimino, always at the same place.
}

fn draw_terrain(win : &mut PistonWindow, e: &Event, terrain: &Terrain){

    for ligne in 0..24{
        for colone in 0..10{
            if terrain[ligne][colone]==0 {continue;}

            let (x_offs, y_offs) = terrain_pixel(ligne as i32, colone as i32);
            win.draw_2d(e,|context,graphics, _device| {
                rectangle([1.0,1.0,1.0,1.0], [x_offs+1.0,y_offs+1.0,33.0,33.0], context.transform, graphics)
                }
            );
        }
    }
}




fn terrain_pixel(ligne:i32, colone:i32)->(f64,f64){
    return ((colone as f64)*35.0 + 465.0, (ligne as f64)*35.0 - 140.0);
}

fn draw_tetrimino(win : &mut PistonWindow, re :&Event, ligne:&i32, colone:&i32, tetrimino:&Tetrimino){
    let(x,y) = terrain_pixel(*ligne, *colone);
    draw_prochain_tetrimino(win,re,x,y,tetrimino);
}

fn draw_prochain_tetrimino(win : &mut PistonWindow,re :&Event,px:f64,py:f64, tetrimino:&Tetrimino){

    for ligne_tetrimino in 0..4 {
        for colone_tetrimino in 0..4{

            if tetrimino.forme[ligne_tetrimino][colone_tetrimino] == 0 {continue;}

            let x_offs = px + 35.0 * colone_tetrimino as f64;
            let y_offs = py +35.0 * ligne_tetrimino as f64;

            win.draw_2d(re,|context, graphics, _device| {

                rectangle(tetrimino.couleur, [x_offs + 1.0, y_offs+1.0, 33.0, 33.0], context.transform, graphics);
                }
            );
        }
    }
}


    fn main() {
    let mut window:PistonWindow = WindowSettings::new("Tetris", [1280,720])
    .exit_on_esc(true)
    .vsync(true)
    .build().unwrap();

    window.events.set_ups(30);

    /*let(_stream, stream_handle)=rodio::OutputStream::try_default().unwrap();
    let music_sink = rodio::Sink::try_new(&stream_handle).unwrap();
    
    music_sink.set_volume(0.1);*/

    let mut blinck_counter = 0;
    let mut sac_depart = creer_sac();
    let mut premier_tetrimino = sac_depart.pop().unwrap();
    let mut second_tetrimino = sac_depart.pop().unwrap();

    let mut etat_jeu = Etat_jeu{
        game_over:false,
        counteur_chute :0,
        terrain: [[0u8;10];24],
        sac : sac_depart,
        tetrimino_actuel : premier_tetrimino,
        prochain_tetrimino: second_tetrimino,
        ligne : 2,
        colone : 3,
        key_map: [false;6]
    };

    while let Some(event) = window.next(){
        match event{

            Event::Loop(Loop::Render(_args_not_used)) =>
            {
                rendu(&mut window, &event, &etat_jeu.ligne, &etat_jeu.colone, &etat_jeu.tetrimino_actuel,&etat_jeu.prochain_tetrimino,&etat_jeu.terrain);
            }

            Event::Loop(Loop::Update(_args_also_not_used))=>{
                if etat_jeu.game_over{
                    if blinck_counter==15{
                        etat_jeu.terrain = [[0u8;10];24];
                    }
                    
                    if blinck_counter==30{
                        etat_jeu.terrain = [[1u8;10];24]
                    }
                    blinck_counter+=1;
                }
                else{
                    mise_a_jour(&mut etat_jeu);

                    /*if etat_jeu.game_over{
                        music_sink.stop();
                    }else {
                        if music_sink.empty(){
                            let music_file = File::open("NESTetrismusic3.ogg").unwrap();
                            let music_source = rodio::Decoder::new(BufReader::new(music_file)).unwrap();                            music_sink.append(music_source);
                            music_sink.play();
                        }
                    }*/
                }
            }

            Event::Input(Input::Button(button_args),_time_stamp )=>{
                if button_args.state == ButtonState::Press{
                    action_touche(&mut etat_jeu.key_map, button_args);
                }
            }

            _=> ()
        }
    }
}
