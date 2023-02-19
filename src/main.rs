use std::{io::{stdout}, time::Duration, path::{PathBuf}};
use clap::Parser;
use crossterm::{execute, cursor::{SavePosition, RestorePosition}, style::Print, terminal::{enable_raw_mode, disable_raw_mode}};
use image::{io::Reader as ImageReader, imageops, GenericImageView};
use serenity::{prelude::GatewayIntents, Client, framework::StandardFramework, model::prelude::{ChannelId}};



#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}
impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue
        }
    }

    pub fn difference_to(&self, other: &Self) -> u32 {
        other.red.abs_diff(self.red) as u32 +
        other.green.abs_diff(self.green) as u32 +
        other.blue.abs_diff(self.blue) as u32
    }

}

#[derive(Parser, Debug)]
struct Options {
    twemoji_path: PathBuf,
    discord_token: String,
    discord_channel: u64,
    #[arg(long, default_value_t = 40)]
    width: u32,
    #[arg(long, default_value_t = 40)]
    height: u32,
    image: PathBuf,
}


#[tokio::main]
async fn main() {
    let args = Options::parse();

    let img = ImageReader::open(args.image).unwrap().decode().unwrap();


    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;


    
    let framework = StandardFramework::new();

    println!("Connecting to discord");
    let client = Client::builder(args.discord_token, intents).framework(framework).await.unwrap();
    println!("Connected");

    let img = imageops::resize(&img, args.width, args.height, imageops::FilterType::Gaussian);


    let mut color_map = Vec::new();




    let mut f = args.twemoji_path.clone();
    f.push("assets/72x72/");

    let files = std::fs::read_dir(&f).unwrap().count();
    println!("Loading {files} emoji files");
    let n = files / 10;
    enable_raw_mode().unwrap();
    execute!(
        stdout(),
        SavePosition
    ).unwrap();
    for (i, path) in std::fs::read_dir(f).unwrap().flatten().enumerate() {

        let p = path.path();

        let p_name = p.file_name().unwrap();

        let img = match ImageReader::open(&p) {
            Ok(v) => v.decode().unwrap(),
            Err(e) => panic!("{e:?}")
        };
        let pixels: Vec<[u8; 3]> = img.pixels().map(|v| [v.2.0[0], v.2.0[1], v.2.0[2]]).collect();
    
        let len = pixels.len();
        let mut sum = [0, 0, 0];
        for v in pixels {
            sum[0] += v[0] as u64;
            sum[1] += v[1] as u64;
            sum[2] += v[2] as u64;
        }
        sum[0] /= len as u64;
        sum[1] /= len as u64;
        sum[2] /= len as u64;
    
    
    
        // let img = imageops::resize(&img, 1, 1, imageops::FilterType::Gaussian);
        // let v = img.pixels().next().unwrap().0;
        let color = Color::new(sum[0] as u8, sum[1] as u8, sum[2] as u8);

        let s = p_name.to_str().unwrap();
        let path = s[..s.len() - 4].to_string();

        let mut final_s = String::new();
        let mut icount = 0;
        for elem in path.split('-') {
            icount += 1;
            final_s.push(char::from_u32(u32::from_str_radix(elem, 16).unwrap()).unwrap());
        }
        if icount < 3 {
            color_map.push((color, final_s));
        }

        if (i % n) == 0 || i == files - 1 {
            let mut s = format!("{:.2}%", (i as f64 / files as f64) * 100.);
            if i == files - 1 {
                s = "100%       ".to_string();
            }
            execute!(
                stdout(),
                RestorePosition,
                Print("Loading emoji - "),
                Print(s)
            ).unwrap();
            if i == files - 1 {
                disable_raw_mode().unwrap();
                println!();
                break;
            }
        }

    
    }


    let mut cur_w = 0;

    //    ChannelId(1056537169143021620).say(&client.cache_and_http.http, "a").await.unwrap();
    let mut output = Vec::new();
    output.push(String::new());


    let pixels = img.pixels().len();
    let n = pixels / 10;

    enable_raw_mode().unwrap();
    execute!(
        stdout(),
        SavePosition
    ).unwrap();
    for (i, pixel) in img.pixels().enumerate() {
        let c = Color::new(pixel.0[0], pixel.0[1], pixel.0[2]);
        let mut current_closest: (String, u32) = ("0".to_owned(), u32::MAX);
        color_map.iter().for_each(|(color, character)| {
            let diff = c.difference_to(color);
            if diff < current_closest.1 {
                current_closest = (character.clone(), diff);
            }
        });
        // println!("For {:?} doing {}", c, current_closest.0);



        output.last_mut().unwrap().push_str(&current_closest.0);
        cur_w += 1;
        if cur_w >= args.width {
            cur_w = 0;
            output.push(String::new());
        }

        if (i % n) == 0 || i == pixels - 1 {
            let mut s = format!("{:.2}%", (i as f64 / pixels as f64) * 100.);
            if i == pixels - 1 {
                s = "100%       ".to_string();
            }
            execute!(
                stdout(),
                RestorePosition,
                Print("Converting image - "),
                Print(s)
            ).unwrap();
            if i == pixels - 1 {
                disable_raw_mode().unwrap();
                println!();
                break;
            }
        }
    }
    const PER_MSG: usize = 5;


    for v in output.as_slice().chunks(PER_MSG) {
        let v = v.join("\n");
        if !v.is_empty() {
            ChannelId(args.discord_channel).say(&client.cache_and_http.http, v).await.unwrap();
            tokio::time::sleep(Duration::from_millis(250)).await;
        } else {
            break;
        }
    }
    

}
