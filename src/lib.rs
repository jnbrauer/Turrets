use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

use ggez::{Context, event, GameError, GameResult, graphics};
use ggez::event::{EventHandler, KeyMods};
use ggez::input::keyboard::KeyCode;
use ggez::timer;

const FPS: u32 = 60;

const SHOT_RADIUS: f32 = 5.0;
const TURRET_RADIUS: f32 = 15.0;
const PLAYER_RADIUS: f32 = 20.0;

/// Point data structure containing X and Y coordinates
#[derive(Clone)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    /// Create a new point with the given coordinates
    fn new(x: f32, y: f32) -> Point {
        return Point { x, y };
    }

    /// Find the linear distance to another point
    fn distance_to(&self, other: &Point) -> f32 {
        // Use the Pythagorean theorem to calculate the distance between the points
        return ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt();
    }

    /// Update the position of this point after moving for a given time at a given velocity
    fn move_time(&mut self, dt: f32, velocity: &Velocity) {
        // Get the X and Y components of the velocity
        let (dx, dy) = velocity.get_components();

        // Multiply the components by the change in time and add to the current position
        self.x += dx * dt;
        self.y += dy * dt;
    }

    /// Move this point a linear distance in a given direction
    fn move_distance(&mut self, distance: f32, heading: f32) {
        // Multiply the XY components of the heading by the distance and add to the current position
        self.x += heading.cos() * distance;
        self.y += heading.sin() * distance;
    }

    /// Check if this point is outside of the given bounds
    fn is_out_of_bounds(&self, bounds: (f32, f32)) -> bool {
        let (max_x, max_y) = bounds;

        return self.x > max_x || self.x < 0.0 || self.y > max_y || self.y < 0.0;
    }

    /// If this point is out of bounds, wrap it to other side of those bounds
    fn wrap_bounds(&mut self, bounds: (f32, f32)) {
        let (max_x, max_y) = bounds;

        if self.x > max_x {self.x = 0.0}
        else if self.x < 0.0 {self.x = max_x}

        if self.y > max_y {self.y = 0.0}
        else if self.y < 0.0 {self.y = max_y}
    }

    /// Prevent this point from going out of bounds
    fn keep_in_bounds(&mut self, bounds: (f32, f32)) {
        let (max_x, max_y) = bounds;

        if self.x > max_x {self.x = max_x}
        else if self.x < 0.0 {self.x = 0.0}

        if self.y > max_y {self.y = max_y}
        else if self.y < 0.0 {self.y = 0.0}
    }
}

/// Velocity data type containing a speed and heading
#[derive(Clone)]
pub struct Velocity {
    speed: f32, // Pixels per second
    heading: f32, // Radians
}

impl Velocity {
    /// Create a new velocity object with the given speed and heading
    fn new(speed: f32, heading: f32) -> Velocity {
        return Velocity { speed, heading };
    }

    /// Get the X and Y components of this velocity
    fn get_components(&self) -> (f32, f32) {
        let x = self.heading.cos() * self.speed;
        let y = self.heading.sin() * self.speed;
        return (x, y);
    }
}

/// Trait specifying the methods an Actor in the game must have
pub trait Actor {
    /// Get the unique ID number of this Actor
    fn get_id(&self) -> u32;
    /// Get the radius of this Actor
    fn get_radius(&self) -> f32;
    /// Get the positions of this Actor
    fn get_position(&self) -> &Point;

    /// Draw this Actor
    fn draw(&self, ctx: &mut Context) -> GameResult;
    /// Update the state of this Actor
    fn update(&mut self, dt: f32);

    /// Check if this Actor has collided with another Actor
    fn check_for_collision(&mut self, other: &Box<dyn Actor>) -> bool {
        // The actors have collided if the distance between them is less than the sum of their radii (minus a tolerance)
        // and their ID's are not equal (they are not the same actor)
        return self.get_position().distance_to(other.get_position()) < (self.get_radius() + other.get_radius() - 0.1)
            && self.get_id() != other.get_id();
    }

    /// Get the amount of damage that this Actor does during a collision
    fn get_damage(&self) -> f32;
    /// Do damage to this Actor
    fn do_damage(&mut self, damage: f32);
    /// Get the new Shots that this Actor has created
    fn collect_shots(&mut self) -> Vec<Shot>;
    /// Check if this Actor is dead
    fn is_dead(&self) -> bool;
}

/// Generate a new unique ID for new Actor
fn get_next_actor_id() -> u32 {
    let id;
    unsafe {
        static mut NEXT: u32 = 0;
        NEXT += 1;
        id = NEXT;
    }
    return id;
}

/// Shot data structure
#[derive(Clone)]
pub struct Shot {
    id: u32,
    position: Point,
    bounds: (f32, f32),
    velocity: Velocity,
    damage: f32,
    health: f32,
}

impl Shot {
    /// Create a new shot with the given starting position, velocity, damage, and lifespan
    fn new(position: Point, bounds: (f32, f32), velocity: Velocity, damage: f32, lifespan: f32) -> Shot {
        return Shot {
            id: get_next_actor_id(),
            position,
            bounds,
            velocity,
            damage,
            health: lifespan * 10.0,
        }
    }
}

impl Actor for Shot {
    /// Get the ID of this Shot
    fn get_id(&self) -> u32 {
        return self.id;
    }

    /// Get the radius of this Shot
    fn get_radius(&self) -> f32 {
        return SHOT_RADIUS;
    }

    /// Get the position of this Shot
    fn get_position(&self) -> &Point {
        return &self.position;
    }

    /// Draw this Shot
    fn draw(&self, ctx: &mut Context) -> GameResult {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0.0, 0.0],
            self.get_radius(),
            0.1,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, ([self.position.x, self.position.y], self.velocity.heading, graphics::WHITE,))?;

        return Ok(());
    }

    /// Update the state of this Shot
    fn update(&mut self, dt: f32) {
        // Move the shot
        self.position.move_time(dt, &self.velocity);
        // Reduce the health of the shot by 10 for every second that passes
        self.health -= dt * 10.0;
    }

    /// Get the amount of damage this Shot does
    fn get_damage(&self) -> f32 {
        return self.damage;
    }

    /// Do damage to this Shot
    fn do_damage(&mut self, damage: f32) {
        self.health -= damage;
    }

    /// Get any new Shots this Shot has created (this will always be an empty vector)
    fn collect_shots(&mut self) -> Vec<Shot> {
        return Vec::new();
    }

    /// Check if this Shot is dead and should be removed
    fn is_dead(&self) -> bool {
        // A shot is dead if the health is below 0 or it has left the game window
        return self.health <= 0.0 || self.position.is_out_of_bounds(self.bounds);
    }
}

/// Turret data structure
#[derive(Clone)]
struct Turret {
    id: u32,
    position: Point,
    bounds: (f32, f32),
    health: f32,
    rotation: f32,
    turn_speed: f32,
    shots: Vec<Shot>,
    time_since_last_shot: f32,
}

impl Turret {
    /// Create a new Turret at the given position with the given bounds
    fn new(position: Point, bounds: (f32, f32)) -> Turret {
        return Turret {
            id: get_next_actor_id(),
            position,
            bounds,
            health: 100.0,
            rotation: 0.0,
            turn_speed: 1.0,
            shots: Vec::new(),
            time_since_last_shot: 0.0,
        };
    }

    /// Fire 4 shots
    fn fire_shots(&mut self) {
        for i in 0..4 {
            // Create the velocity of the new shot and rotate it 90 degrees * i
            let mut shot_velocity = Velocity::new(200.0, self.rotation);
            shot_velocity.heading += i as f32 * (PI/2.0);

            // Initialize the position of the shot and move it away fro the turret
            let mut shot_position = self.position.clone();
            shot_position.move_distance(self.get_radius() + SHOT_RADIUS, shot_velocity.heading);

            // Create the shot
            let shot = Shot::new(
                shot_position,
                self.bounds,
                shot_velocity,
                25.0,
                3.0,
            );

            // Add the shot to the list of shots
            self.shots.push(shot);
        }
    }
}

impl Actor for Turret {
    /// Get the ID of this Turret
    fn get_id(&self) -> u32 {
        return self.id;
    }

    /// Ge the radius of this Turret
    fn get_radius(&self) -> f32 {
        return TURRET_RADIUS;
    }

    /// Get the position of this Turret
    fn get_position(&self) -> &Point {
        return &self.position;
    }

    /// Draw this Turret
    fn draw(&self, ctx: &mut Context) -> GameResult {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0.0, 0.0],
            self.get_radius(),
            5.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, ([self.position.x, self.position.y], self.rotation, graphics::WHITE,))?;

        return Ok(());
    }

    /// Update the state of this Turret
    fn update(&mut self, dt: f32) {
        // Rotate the turret
        self.rotation += dt * self.turn_speed;

        // If enough time has elapsed since the last shot, fire again
        if self.time_since_last_shot > 2.0 {
            self.fire_shots();
            self.time_since_last_shot = 0.0;
        } else {
            self.time_since_last_shot += dt;
        }
    }

    /// Get the amount of damage that hitting this Turret causes
    fn get_damage(&self) -> f32 {
        return 100.0;
    }

    /// Do damage to this Turret
    fn do_damage(&mut self, damage: f32) {
        self.health -= damage;
    }

    /// Get the new shots this Turret has created since last shot collection
    fn collect_shots(&mut self) -> Vec<Shot> {
        // Copy the list of new shots
        let shots_copy = self.shots.clone();
        // Clear the list of shots of the turret
        self.shots.clear();
        // Return the cloned list
        return shots_copy;
    }

    /// Check if this Turret is dead
    fn is_dead(&self) -> bool {
        // Turret is dead if its health goes below 0
        return self.health <= 0.0;
    }
}

/// Player data structure
#[derive(Clone)]
struct Player {
    id: u32,
    position: Point,
    bounds: (f32, f32),
    health: f32,
    velocity: Velocity,
    shots: Vec<Shot>,
    current_pressed_key: KeyCode,
}

impl Player {
    /// Create a new Player at the given position with the given bounds
    fn new(position: Point, bounds: (f32, f32)) -> Player {
        return Player {
            id: get_next_actor_id(),
            position,
            bounds,
            health: 100.0,
            velocity: Velocity::new(0.0, 0.0),
            shots: Vec::new(),
            current_pressed_key: KeyCode::Delete,
        };
    }

    /// Fire a shot out the front of the Player
    fn fire_shot(&mut self) {
        // Clone the velocity of the player and 200 to the speed to use as the speed of the shot
        let mut shot_velocity = self.velocity.clone();
        shot_velocity.speed += 200.0;

        // Clone the position of the player and move it away from the player to use as the position of the shot
        let mut shot_position = self.position.clone();
        shot_position.move_distance(self.get_radius() + SHOT_RADIUS, shot_velocity.heading);

        // Initialize the shot
        let shot = Shot::new(
            shot_position,
            self.bounds,
            shot_velocity,
            20.0,
            5.0,
        );

        // Add the shot to the list of shots
        self.shots.push(shot);
    }

    /// Handle a key down event
    fn handle_key_down_event(&mut self, keycode: KeyCode, repeat: bool) {
        match keycode {
            // If the up arrow is pressed, move forwards
            KeyCode::Up => {
                self.velocity.speed = 150.0;
            }
            // If the down arrow is pressed, move backwards
            KeyCode::Down => {
                self.velocity.speed = -150.0;
            }
            // If the spacebar is pressed, fire a shot
            KeyCode::Space => {
                if !repeat {
                    self.fire_shot();
                }
            }
            // If any other key is pressed, track what key is currently pressed
            _ => {
                self.current_pressed_key = keycode;
            }
        }
    }

    /// Handle a key up event
    fn handle_key_up_event(&mut self, keycode: KeyCode) {
        match keycode {
            // If either the up arrow or the down arrow is released, stop moving
            KeyCode::Up | KeyCode::Down => {
                self.velocity.speed = 0.0;
            }
            // If any other key is pressed, track what key is currently pressed
            _ => {
                // If the released key was the last key to be pressed down (other than up down or space),
                // reset the current key to delete (placeholder for no key)
                if keycode == self.current_pressed_key {
                    self.current_pressed_key = KeyCode::Delete;
                }
            }
        }
    }
}

impl Actor for Player {
    /// Get the ID of this Player
    fn get_id(&self) -> u32 {
        return self.id;
    }

    /// Get the radius of this Player
    fn get_radius(&self) -> f32 {
        return PLAYER_RADIUS;
    }

    /// Get the position of this Player
    fn get_position(&self) -> &Point {
        return &self.position;
    }

    /// Draw this Player
    fn draw(&self, ctx: &mut Context) -> GameResult {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0.0, 0.0],
            self.get_radius(),
            5.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, ([self.position.x, self.position.y], self.velocity.heading, graphics::WHITE,))?;

        return Ok(());
    }

    /// Update the state of this Player
    fn update(&mut self, dt: f32) {
        match self.current_pressed_key {
            // If the right arrow key is being held down, turn right
            KeyCode::Right => {
                self.velocity.heading += 0.05;
            }
            // If the left arrow key is being held down, turn left
            KeyCode::Left => {
                self.velocity.heading -= 0.05;
            }
            _ => ()
        }

        // Move the player
        self.position.move_time(dt, &self.velocity);
        // Prevent the player from leaving the bounds of the window
        self.position.keep_in_bounds(self.bounds);
    }

    /// Get the damage the Player does when collided with
    fn get_damage(&self) -> f32 {
        return 100.0;
    }

    /// Do damage to this Player
    fn do_damage(&mut self, damage: f32) {
        self.health -= damage;
    }

    /// Get the new shots this Player has created since last shot collection
    fn collect_shots(&mut self) -> Vec<Shot> {
        // Copy the list of new shots
        let shots_copy = self.shots.clone();
        // Clear the list of shots of the player
        self.shots.clear();
        // Return the cloned list
        return shots_copy;
    }

    /// Check if this player is dead
    fn is_dead(&self) -> bool {
        // The player is dead if health goes below 0
        return self.health <= 0.0;
    }
}

/// Data structure to store the main state of the game
pub struct MainState {
    player: Player,
    actors: Vec<Box<dyn Actor>>,
}

impl MainState {
    /// Initialize the state of the game
    pub fn new(ctx: &Context) -> MainState {
        // Get the size of the window
        let bounds = graphics::drawable_size(ctx);
        let (width, height) = bounds;

        // Initialize a new MainState object
        let mut state = MainState {
            // Initialize the Player
            player: Player::new(Point::new(width/2.0, height/2.0), bounds),
            // Initialize a vector to hold the actors in the game
            actors: Vec::new(),
        };

        // Create 4 turrets and add them to the game
        state.add_actor(Box::new(Turret::new(Point::new(width/4.0, height/4.0), bounds)));
        state.add_actor(Box::new(Turret::new(Point::new(width/4.0, height*0.75), bounds)));
        state.add_actor(Box::new(Turret::new(Point::new(width*0.75, height/4.0), bounds)));
        state.add_actor(Box::new(Turret::new(Point::new(width*0.75, height*0.75), bounds)));

        return state;
    }

    /// Add an actor to the game
    fn add_actor(&mut self, actor: Box<dyn Actor>) {
        self.actors.push(actor);
    }

    /// Collect any new shots created by any actor
    fn collect_shots(&mut self) {
        // Create a vector to hold all of the new shots
        let mut new_shots: Vec<Shot> = Vec::new();

        // Collect the shots from the player and add them to the list of shots
        new_shots.append(&mut self.player.collect_shots());

        // Collect the shots from all the other actors and add them to the list of shots
        for actor in &mut self.actors {
            new_shots.append(&mut actor.collect_shots());
        }

        // Add all the shots to the game
        for shot in new_shots {
            self.add_actor(Box::new(shot));
        }
    }

    /// Handle collision between all of the actors
    fn handle_collisions(&mut self) {
        // Loop through all of the actors in the game
        for i in 0..self.actors.len() {
            // Get the list of actors after the current actor in the list
            let (head, tail) = self.actors.split_at_mut(i+1);
            // Get a reference to the current actors
            let actor = &mut head[i];

            // Check if the current actor has collided with the player
            if self.player.check_for_collision(actor) {
                // If it has, do damage to the player and the actor
                self.player.do_damage(actor.get_damage());
                actor.do_damage(self.player.get_damage());
            }

            // Loop over the remaining actors in the list
            for j in 0..tail.len() {
                // Get a reference to the next actor in the list
                let other_actor = &mut tail[j];
                // Check if the two actors have collided
                if actor.check_for_collision(other_actor) {
                    // If they have, do damage to both actors
                    actor.do_damage(other_actor.get_damage());
                    other_actor.do_damage(actor.get_damage());
                }
            }
        }
    }

    /// Remove the dead actors from the game
    fn remove_dead(&mut self) {
        // Only keep the actors that are not dead in the list of actors
        self.actors.retain(|actor| !actor.is_dead());
    }
}

impl EventHandler for MainState {
    /// Update the MainState
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, FPS) {
            // Update the state of the player
            self.player.update(1.0 / FPS as f32);
            // Update the state of every actor
            for actor in &mut self.actors {
                actor.update(1.0 / FPS as f32);
            }

            // Collect shots
            self.collect_shots();
            // Handle collisions
            self.handle_collisions();
            // Remove dead actors
            self.remove_dead();

            // If the player has died, end the game
            if self.player.is_dead() {
                event::quit(ctx);
            }
        }

        return Ok(());
    }

    /// Draw the game
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Clear the canvas
        graphics::clear(ctx, graphics::BLACK);

        // Draw the player
        self.player.draw(ctx)?;
        // Draw all the actors
        for actor in &self.actors {
            actor.draw(ctx)?;
        }

        // Show the game to the user
        graphics::present(ctx)?;

        timer::yield_now();

        return Ok(());
    }

    /// Handle key down event
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, repeat: bool) {
        // If escape is pressed, end the game
        if keycode == KeyCode::Escape {
            event::quit(ctx);
        }
        // Forward the key event to the player object
        self.player.handle_key_down_event(keycode, repeat);
    }

    /// Handle key up event
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        // Forward the key event to the player object
        self.player.handle_key_up_event(keycode);
    }
}
