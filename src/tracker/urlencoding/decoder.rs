pub fn from_url(url: Vec<u8>) -> Vec<u8> {
    //Este nombre es para seguir la convencion que teniamos en el tp viejo
    let mut counter = 0;
    let mut hex = String::new();
    let mut vec_res = vec![];
    for byte in url {
        if byte == b'%' {
            counter = 2;
            continue;
        } else if counter > 0 {
            hex.push(byte as char);
            counter -= 1;
            if counter == 0 {
                if let Ok(num) = u8::from_str_radix(&hex, 16) {
                    vec_res.push(num)
                };
                hex = String::new();
            };
        } else {
            vec_res.push(byte);
        }
    }
    vec_res
}
