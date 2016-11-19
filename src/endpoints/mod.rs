mod request;
mod assets;
mod collections;
mod episodes;
mod franchises;
mod seasons;
mod shows;
mod specials;

pub use endpoints::assets::AssetEndpoint;
pub use endpoints::collections::CollectionEndpoint;
pub use endpoints::episodes::EpisodeEndpoint;
pub use endpoints::franchises::FranchiseEndpoint;
pub use endpoints::seasons::SeasonEndpoint;
pub use endpoints::shows::ShowEndpoint;
pub use endpoints::specials::SpecialEndpoint;
