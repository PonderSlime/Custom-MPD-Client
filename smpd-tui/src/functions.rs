extern crate mpd;

use mpd::Client;

use regex::Regex;


pub fn process_queue(mut conn: Client) -> Vec<String> {
    let mut list_items = vec![];

    let queue_result = conn.queue();

    match queue_result {
        Ok(songs) => {
            //println!("Queue loaded successfully. Contains {} songs.", songs.len());

            for song in songs {
                let title = song.title.as_deref().unwrap_or("No Title");
                let artist = song.artist.as_deref().unwrap_or("No Artist");
                let queue_place = format!("{:?}", &song.place);
                let mut queue_place_pos = String::from("");

                let re = Regex::new(r"pos: (\d+)").unwrap();
                if let Some(caps) = re.captures(&queue_place) { 
                    if let Some(pos_str) = caps.get(1) {
                        if let Ok(pos_val) = pos_str.as_str().parse::<i32>() {
                            //println!("Pos val: {:?}", pos_val);
                            queue_place_pos = pos_val.to_string();
                        }
                    }
                }
                //println!("---");
                //println!("  Title: {}", title);
                //println!("  Artist: {}", artist);
                //println!("  Pos: {}", queue_place_pos);
                list_items.push(format!("{}: {} - {}", queue_place_pos, artist, title));

            }
        }
        Err(e) => {
            eprintln!("Failed to get queue: {:?}", e);
        }
    }

    return list_items;
}

