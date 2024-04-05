use std::{io::{stdout, Stdout, Write}, time::Duration, vec};
use std::{thread, time};
use rand::Rng;
use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{self, poll,read, Event, KeyCode}, execute, style::{ Print}, terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType}, ExecutableCommand, QueueableCommand
};


struct Location{
    c: u16,
    l: u16
}
struct Enemy{
    location: Location
}

struct World{
    player_location: Location,
    maxc: u16,
    maxl:u16,
    map: Vec<(u16,u16)>,
    is_died: bool,
    next_start: u16,
    next_end: u16,
    enemies: Vec<Enemy>
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
    scr.queue(MoveTo(world.player_location.c,world.player_location.l))?;
    scr.queue(Print("P"))?;

    //draw enemies
    for e in 0..world.enemies.len() {
        scr.queue(MoveTo(world.enemies[e].location.c, world.enemies[e].location.l))?;
        scr.queue(Print("E"))?;
    }

    scr.flush()?;
    Ok(())
}

fn physics(mut world: World) -> std::io::Result<World> {

    if world.player_location.c >= world.map[world.player_location.l as usize].1 || world.player_location.c <= world.map[world.player_location.l as usize].0 {
        world.is_died = true;
    }


    let mut rng = rand::thread_rng();
    for l in (0..world.map.len() - 1).rev() {
        world.map[l + 1] = world.map[l];
    }

    if rng.gen_range(0..10) >= 7{
        if world.next_end < world.map[0].1 {
            world.map[0].1 -= 1;
        }
        if world.next_end > world.map[0].1 {
            world.map[0].1 += 1;
        }
        if world.next_start < world.map[0].0 {
            world.map[0].0 -= 1;
        }
        if world.next_start > world.map[0].0 {
            world.map[0].0 += 1;
        }
        if world.next_end == world.map[0].1 && world.next_start == world.map[0].0 {
            world.next_start = rng.gen_range(world.map[0].0 - 5..world.map[0].1 - 5);
            world.next_end = rng.gen_range(world.map[0].0 + 5..world.map[0].1 + 5);
            if world.next_end - world.next_start <= 7 {
                world.next_start -= 7;
            }
        }
    }

    if rng.gen_range(0..10) >= 9 {
        world.enemies.push(Enemy { location: Location { c: rng.gen_range(world.map[0].0..world.map[0].1), l: 0 } })
    }

    for e in (0..world.enemies.len()).rev() {
        world.enemies[e].location.l += 1;
        if world.enemies[e].location.l == world.maxl {
            world.enemies.remove(e);
        }
    }


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
        player_location: Location{
            c:max_c/2,
            l:max_l - 1,
        },
        maxc: max_c,
        maxl: max_l,
        map: vec![((max_c/2) -5 , (max_c/2)+5);max_l as usize],
        is_died: false,
        next_start: (max_c/2) - 10,
        next_end:  (max_c/2) + 10,
        enemies: vec![]
    };

    while  !world.is_died {
        
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
                                if world.player_location.l > 1 {
                                    world.player_location.l -= 1;
                                }
                            }
                            KeyCode::Char('s') => {
                                if world.player_location.l < max_l - 1 {
                                    world.player_location.l += 1;
                                }
                            }
                            KeyCode::Char('d') => {
                                if world.player_location.c < max_c - 1 {
                                    world.player_location.c += 1;
                                }
                            }
                            KeyCode::Char('a') => {
                                if world.player_location.c > 1 {
                                    world.player_location.c -= 1;
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