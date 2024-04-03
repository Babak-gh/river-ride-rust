use std::{io::{stdout, Stdout, Write}, time::Duration, vec};
use std::{thread, time};

use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{self, poll,read, Event, KeyCode}, execute, style::{ Print}, terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType}, ExecutableCommand, QueueableCommand
};

struct World{
    player_c:u16,
    player_l:u16,
    maxc: u16,
    maxl:u16,
    map: Vec<(u16,u16)>,
    is_died: bool
}

fn draw(mut scr: &Stdout, mut world: &World) -> std::io::Result<()>{
    scr.queue(Clear(ClearType::All))?;

    //draw map
    for l in 0..world.map.len() {
        scr.queue(MoveTo(0, l as u16))?;
        scr.queue(Print("+".repeat(world.map[l].0 as usize)))?;
        scr.queue(MoveTo(world.map[l].1, l as u16))?;
        scr.queue(Print("+".repeat((world.maxc - world.map[l].1) as usize)))?;
    }

    // draw player
    scr.queue(MoveTo(world.player_c,world.player_l))?;
    scr.queue(Print("P"))?;

    scr.flush()?;
    Ok(())
}

fn physics(mut world: World) -> std::io::Result<World> {

    if world.player_c >= world.map[world.player_l as usize].1 || world.player_c <= world.map[world.player_l as usize].0 {
        print!("Fuck");
        world.is_died = true
    }


    for l in (0..world.map.len() - 1).rev() {
        world.map[l + 1] = world.map[l];
    }
    world.map[0] = (20 , 80);

    Ok(world)
}


fn main() -> std::io::Result<()> {

    // init the screen
    let mut scr = stdout();
    scr.execute(Hide)?;
    let (max_c,max_l) = size().unwrap();
    enable_raw_mode()?;

    // init the game
    let mut world = World{
        player_c : max_c/2,
        player_l : max_l - 1,
        maxc: max_c,
        maxl: max_l,
        map: vec![((max_c/2) -5 , (max_c/2)+5);max_l as usize],
        is_died: false
    };

    while  !world.is_died {
            // `poll()` waits for an `Event` for a given time period
            if poll(Duration::from_millis(10))? {
                let key = read().unwrap();
                while poll(Duration::from_millis(0)).unwrap() {
                    _ = read();
                }
                match key {
                    Event::Key(event) => {
                        match event.code {
                            KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Char('w') => {
                                if world.player_l > 1 {
                                    world.player_l -= 1;
                                }
                            }
                            KeyCode::Char('s') => {
                                if world.player_l < max_l - 1 {
                                    world.player_l += 1;
                                }
                            }
                            KeyCode::Char('d') => {
                                if world.player_c < max_c - 1 {
                                    world.player_c += 1;
                                }
                            }
                            KeyCode::Char('a') => {
                                if world.player_c > 1 {
                                    world.player_c -= 1;
                                }
                            }
                            _ => {}
                        }
                    },
                    _ => {},
                }
            } else {
                // Timeout expired and no `Event` is available
            }
       

        world = physics(world).unwrap();
        

        draw(&scr, &world)?;
        thread::sleep(time::Duration::from_millis(100));
    }

    scr.execute(Show)?;
    disable_raw_mode()?;
    scr.execute(Clear(ClearType::All))?;
    scr.execute(Print("Thanks for Playing"))?;
    
    Ok(())
}