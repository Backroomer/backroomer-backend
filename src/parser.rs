use mongodb::bson::DateTime;
use regex::Regex;
use scraper::{selectable::Selectable, ElementRef};
use crate::{error::{ParseElementError, WikidotError}, mongo_user::{USER_ADD, USER_NOW}, selectors, user::User};

pub fn odate(element: ElementRef) -> Option<DateTime>{
    let ele_val = if element.value().classes().collect::<Vec<&str>>().contains(&"odate"){
        element
    }
    else{
        element.select(&selectors::ODATE).next()?
    };

    for class in ele_val.value().classes(){
        if class.contains("time_"){
            return Some(DateTime::from_millis(class.replace("time_", "").parse::<i64>().ok()? * 1000))
        }
    }
    None
}

pub fn printuser(element: ElementRef) -> Result<User, WikidotError>{
    let ele_val;

    if element.value().classes().collect::<Vec<&str>>().contains(&"printuser"){
        ele_val = element;
    }
    else{
        if let Some(printuser) = element.select(&selectors::PRINTUSER).next(){
            ele_val = printuser;
        }
        else {return Ok(User::from_deleted_user(None))}
    }

    if ele_val.value().classes().collect::<Vec<&str>>().contains(&"deleted"){
        let user_id = ele_val.attr("data-id")
                .ok_or(ParseElementError::parser_id())?
                .parse::<i32>()?;
        user_add(user_id);

        Ok(User::from_deleted_user(Some(user_id)))
    }
    else if element.text().collect::<String>() == "Wikidot"{
        Ok(User::from_wikidot_user())
    }
    else{
        let id_re = Regex::new(r"\((\d+)\)")?;
        let mut a_eles = element.select(&selectors::A);

        if let Some(a_ele_2) = a_eles.nth(1){
            let user_id = id_re.captures(a_ele_2.attr("onclick").ok_or(ParseElementError::parser_id())?)
                    .ok_or(ParseElementError::parser_id())?
                    .get(1).ok_or(ParseElementError::parser_id())?
                    .as_str().parse::<i32>()?;
            user_add(user_id);
            
            Ok(User::from(
                user_id,
                a_ele_2.text().collect::<String>(),
                a_ele_2.attr("href")
                    .ok_or(ParseElementError::parser_unix_name())?
                    .replace("http://www.wikidot.com/user:info/", "")
            ))
        }
        else{
            Ok(User::from_guest_user(element.text().collect::<String>()))
        }
    }
}

fn user_add(user_id: i32){
    unsafe {
        if !USER_ADD.contains(&user_id) & !USER_NOW.contains(&user_id){
            USER_ADD.push(user_id);
        }
    }
}