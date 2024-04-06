use std::{io::{stdout, Stdout, Write}, time::Duration, vec};
use std::{thread, time};
use env_logger::{Builder, Target};
use rand::Rng;
use log::{debug, warn};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{self, poll,read, Event, KeyCode}, execute, queue, style::Print, terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType}, ExecutableCommand, QueueableCommand
};


struct Bullet{
    location: Location,
    energy: u16
}
struct Location{
    c: u16,
    l: u16
}
struct Enemy{
    location: Location
}

struct Fuel{
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
    enemies: Vec<Enemy>,
    bullets: Vec<Bullet>,
    fuel: u16,
    score: u16,
    edible: Vec<Fuel>
}

fn conflict_locations(first:&Location, second:&Location) -> bool {
    if first.l == second.l && first.c == second.c {
        return true;
    }
    else {
        return false;
    }
    
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

    scr.queue(MoveTo(2,2))?
    .queue(Print(format!("Score: {}", world.score)))?
    .queue(MoveTo(2,3))?
    .queue(Print(format!("Fuel: {}", world.fuel)))?;

    // draw player
    scr.queue(MoveTo(world.player_location.c,world.player_location.l))?;
    scr.queue(Print("P"))?;

    //draw enemies
    for e in 0..world.enemies.len() {
        scr.queue(MoveTo(world.enemies[e].location.c, world.enemies[e].location.l))?;
        scr.queue(Print("E"))?;
    }

    //draw fuel
    for f in 0..world.edible.len() {
        scr.queue(MoveTo(world.edible[f].location.c, world.edible[f].location.l))?;
        scr.queue(Print("$"))?;
    }

    //draw bullet
    for b in &world.bullets  {
        scr.queue(MoveTo(b.location.c, b.location.l))?;
        scr.queue(Print("*"))?;
    }

    scr.flush()?;
    Ok(())
}

fn physics(mut world: World) -> std::io::Result<World> {


    if world.player_location.c >= world.map[world.player_location.l as usize].1 || world.player_location.c <= world.map[world.player_location.l as usize].0 {
        world.is_died = true;
    }

    world.fuel -= 1;
    if world.fuel == 0 {
        world.is_died = true
    }

    for  e in (0..world.enemies.len()).rev()  {
        if conflict_locations(&world.player_location , &world.enemies[e].location) {
            world.is_died = true;
        }
        for  b in (0..world.bullets.len()).rev()  {
            if conflict_locations(&world.bullets[b].location, &world.enemies[e].location) ||  conflict_locations(&Location { c: world.bullets[b].location.c, l: world.bullets[b].location.l - 1 }, &world.enemies[e].location){
                world.enemies.remove(e);
                world.score += 10
            }
        }
    }

    for f in (0..world.edible.len()).rev() {
        if conflict_locations(&world.player_location, &world.edible[f].location) {
            world.fuel += 20;
            world.edible.remove(f);
        }
    }

    for  b in (0..world.bullets.len()).rev()  {
        world.bullets[b].energy -= 1;
        if world.bullets[b].energy <= 0 {
            world.bullets.remove(b);
            break;
        }
       world.bullets[b].location.l -= 2;
       
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

    if rng.gen_range(0..10) >= 9 {
        world.edible.push(Fuel { location: Location { c: rng.gen_range(world.map[0].0..world.map[0].1), l: 0 } })
    }

    for e in (0..world.enemies.len()).rev() {
        world.enemies[e].location.l += 1;
        if world.enemies[e].location.l == world.maxl {
            world.enemies.remove(e);
        }
    }

    for e in (0..world.edible.len()).rev() {
        world.edible[e].location.l += 1;
        if world.edible[e].location.l == world.maxl {
            world.edible.remove(e);
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
        enemies: vec![],
        bullets: vec![],
        fuel: 100,
        score: 0,
        edible: vec![]
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
                            KeyCode::Char(' ') => {
                                if world.bullets.len() == 0 {
                                    world.bullets.push(Bullet { location: Location { c: world.player_location.c, l: world.player_location.l }, energy: world.maxl / 2 })
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