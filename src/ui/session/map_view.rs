use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;
use shumate::prelude::*;

use crate::model;

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/session/map_view.ui")]
    pub(crate) struct MapView {
        #[template_child]
        pub(super) map_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) node_info_label: TemplateChild<gtk::Label>,
        pub(super) shumate_map: RefCell<Option<shumate::SimpleMap>>,
        pub(super) marker_layer: RefCell<Option<shumate::MarkerLayer>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MapView {
        const NAME: &'static str = "PaplMapView";
        type Type = super::MapView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("map.zoom-in", None, move |obj, _, _| {
                obj.zoom_in();
            });
            klass.install_action("map.zoom-out", None, move |obj, _, _| {
                obj.zoom_out();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MapView {
        fn constructed(&self) {
            self.parent_constructed();

            let map = shumate::SimpleMap::new();
            let registry = shumate::MapSourceRegistry::with_defaults();

            // Use OpenStreetMap as default tile source
            if let Some(source) = registry.by_id(shumate::MAP_SOURCE_OSM_MAPNIK) {
                map.set_map_source(Some(&source));
            }

            let viewport = map.viewport().unwrap();
            viewport.set_zoom_level(5.0);

            // Create marker layer
            let marker_layer = shumate::MarkerLayer::new(&viewport);
            map.add_overlay_layer(&marker_layer);

            self.map_container.append(&map);
            self.shumate_map.replace(Some(map));
            self.marker_layer.replace(Some(marker_layer));
        }
    }

    impl WidgetImpl for MapView {}
    impl BinImpl for MapView {}
}

glib::wrapper! {
    pub(crate) struct MapView(ObjectSubclass<imp::MapView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl MapView {
    pub(crate) fn set_device(&self, device: &model::Device) {
        self.update_markers(device);

        // Update markers when nodes change
        device.nodes().connect_items_changed(
            clone!(@weak self as obj, @weak device => move |_, _, _, _| {
                obj.update_markers(&device);
            }),
        );

        // Update markers when waypoints change
        device.waypoints().connect_items_changed(
            clone!(@weak self as obj, @weak device => move |_, _, _, _| {
                obj.update_markers(&device);
            }),
        );
    }

    fn update_markers(&self, device: &model::Device) {
        let imp = self.imp();
        let Some(marker_layer) = imp.marker_layer.borrow().as_ref().cloned() else {
            return;
        };

        // Remove existing markers
        marker_layer.remove_all();

        let mut has_position = false;
        let mut center_lat = 0.0f64;
        let mut center_lon = 0.0f64;
        let mut count = 0u32;

        // Add node markers
        let nodes = device.nodes();
        for i in 0..nodes.n_items() {
            let Some(obj) = nodes.item(i) else { continue };
            let node = obj.downcast_ref::<model::Node>().unwrap();

            let lat = node.imp_lat();
            let lon = node.imp_lon();
            if lat == 0.0 && lon == 0.0 {
                continue;
            }

            has_position = true;
            center_lat += lat;
            center_lon += lon;
            count += 1;

            let label_text = node.short_name();
            let display = if label_text.is_empty() {
                format!("!{:04x}", node.num() & 0xFFFF)
            } else {
                label_text
            };

            let label = gtk::Label::new(Some(&display));
            label.add_css_class("osd");
            label.add_css_class("caption");
            label.set_margin_start(2);
            label.set_margin_end(2);

            let marker = shumate::Marker::new();
            marker.set_child(Some(&label));
            marker.set_location(lat, lon);
            marker_layer.add_marker(&marker);
        }

        // Add waypoint markers
        let waypoints = device.waypoints();
        for i in 0..waypoints.n_items() {
            let Some(obj) = waypoints.item(i) else { continue };
            let wp = obj.downcast_ref::<model::Waypoint>().unwrap();

            let lat = wp.latitude();
            let lon = wp.longitude();
            if lat == 0.0 && lon == 0.0 {
                continue;
            }

            has_position = true;
            center_lat += lat;
            center_lon += lon;
            count += 1;

            let label = gtk::Label::new(Some(&format!("WP: {}", wp.name())));
            label.add_css_class("osd");
            label.add_css_class("caption");
            label.set_margin_start(2);
            label.set_margin_end(2);

            let marker = shumate::Marker::new();
            marker.set_child(Some(&label));
            marker.set_location(lat, lon);
            marker_layer.add_marker(&marker);
        }

        // Center map on markers
        if has_position && count > 0 {
            center_lat /= count as f64;
            center_lon /= count as f64;

            if let Some(map) = imp.shumate_map.borrow().as_ref() {
                let viewport = map.viewport().unwrap();
                viewport.set_zoom_level(12.0);
                map.map().unwrap().center_on(center_lat, center_lon);
            }
        }

        imp.node_info_label.set_label(&format!(
            "{} nodes, {} waypoints on map",
            count.saturating_sub(waypoints.n_items()),
            waypoints.n_items()
        ));
    }

    fn zoom_in(&self) {
        if let Some(map) = self.imp().shumate_map.borrow().as_ref() {
            let vp = map.viewport().unwrap();
            vp.set_zoom_level(vp.zoom_level() + 1.0);
        }
    }

    fn zoom_out(&self) {
        if let Some(map) = self.imp().shumate_map.borrow().as_ref() {
            let vp = map.viewport().unwrap();
            vp.set_zoom_level((vp.zoom_level() - 1.0).max(1.0));
        }
    }
}

/// Helper trait to access private fields for position without exposing them as properties
trait NodePositionExt {
    fn imp_lat(&self) -> f64;
    fn imp_lon(&self) -> f64;
}

impl NodePositionExt for model::Node {
    fn imp_lat(&self) -> f64 {
        // Access through the GObject - we stored lat/lon in imp
        // Use a workaround: expose through a method on Node
        self.latitude()
    }
    fn imp_lon(&self) -> f64 {
        self.longitude()
    }
}
