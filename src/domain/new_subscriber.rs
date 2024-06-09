use crate::routes::Subscriber;

use super::{
    subscriber_email::{InvalidEmail, SubscriberEmail},
    subscriber_name::{InvalidNameError, SubscriberName},
};

pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}

impl TryFrom<Subscriber> for NewSubscriber {
    type Error = InvalidSubscriber;

    fn try_from(subscriber: Subscriber) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::try_from(subscriber.email)?;
        let name = SubscriberName::try_from(subscriber.name)?;
        Ok(NewSubscriber { name, email })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InvalidSubscriber {
    #[error(transparent)]
    InvalidEmail(#[from] InvalidEmail),
    #[error(transparent)]
    InvalidName(#[from] InvalidNameError),
}
