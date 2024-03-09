use crate::routes::Subscriber;

use super::{
    subscriber_email::SubscriberEmail, subscriber_name::SubscriberName,
};

pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}

impl TryFrom<Subscriber> for NewSubscriber {
    type Error = String;

    fn try_from(subscriber: Subscriber) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::try_from(subscriber.email)?;
        let name = SubscriberName::try_from(subscriber.name)?;
        Ok(NewSubscriber { name, email })
    }
}
