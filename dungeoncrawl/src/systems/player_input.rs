use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(Enemy)]
#[write_component(Health)]
pub fn player_input(
    ecs: &mut SubWorld,    
    commands: &mut CommandBuffer,
    #[resource] key: &Option<VirtualKeyCode>,    
    #[resource] turn_state: &mut TurnState
) {        

    if let Some(key) = key {

        if *key == VirtualKeyCode::Space {
            let mut players = <(Entity, &Point)>::query().filter(component::<Player>());
            let player_entity = players.iter(ecs).find_map(|(entity, _)| Some(*entity)).unwrap();
            
            if let Ok(mut health) = ecs.entry_mut(player_entity).unwrap().get_component_mut::<Health>() {         
                health.current = i32::min(health.max, health.current+1);
            }

            *turn_state = TurnState::PlayerTurn;
            return;
        }

        let delta = match key {
            VirtualKeyCode::A => Point::new(-1, 0),
            VirtualKeyCode::D => Point::new(1, 0),
            VirtualKeyCode::W => Point::new(0, -1),
            VirtualKeyCode::S => Point::new(0, 1),
            _ => Point::new(0, 0),
        };

        if delta.x != 0 || delta.y != 0 {
            let mut players = <(Entity, &Point)>::query().filter(component::<Player>());
            let (player_entity, destination) = players.iter(ecs).find_map(|(entity, pos)| Some((*entity, *pos + delta))).unwrap();

            let mut enemies = <(Entity, &Point)>::query().filter(component::<Enemy>());
            
            let mut hit_something = false;
            enemies.iter(ecs).filter(|(_, pos)| { **pos == destination}).for_each(|(entity, _)| {
                hit_something = true;

                commands.push(((), WantsToAttack {
                    attacker: player_entity,
                    victim: *entity,
                }));
            });


            if !hit_something {
                commands.push(((), WantsToMove {
                    entity: player_entity,
                    destination
                }));
            }

            *turn_state = TurnState::PlayerTurn;
        }
    }
}
