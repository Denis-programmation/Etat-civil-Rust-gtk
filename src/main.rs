use gtk::{ListStore, prelude::*};
use gtk::{gio, glib::{BoxedAnyObject}, ApplicationWindow, Button, Entry, Orientation};
use mysql::*;
use mysql::prelude::*;

mod grid_cell;

use crate::grid_cell::{Entry as Entry2, GridCell};
struct Row 
{
    col1: String,
    col2: String,
    col3: String
}

use std::cell::Ref;

fn main() 
{
    // Create a new application
    let application = gtk::Application::builder()
        .application_id("etat-civil2")
        .build();
    application.connect_startup(on_startup);
    application.connect_activate(on_activate);
    application.run();
}

fn on_startup(app: &gtk::Application)
{
// menus
    let about = gio::ActionEntry::builder("about")
        .activate(|_, _, _| println!("A propos"))
        .build();

    let quit = gio::ActionEntry::builder("quit")
        .activate(|app: &gtk::Application, _, _| app.quit())
        .build();

    app.add_action_entries([about, quit]);

    let vue_totale = gio::ActionEntry::builder("vue_totale")
        .activate(|_, _, _| vue_totale() )
        .build();

    let vue_partielle = gio::ActionEntry::builder("vue_partielle")
        .activate(|_, _, _| vue_partielle() )
        .build();

    let ajouter = gio::ActionEntry::builder("ajouter")
        .activate(|_, _, _| ajouter_individu() )
        .build();

    let supprimer = gio::ActionEntry::builder("supprimer")
        .activate(|_, _, _| supprimer_individu() )
        .build();

    let reset = gio::ActionEntry::builder("reset")
        .activate(|_, _, _| reset_database() )
        .build();

    app.add_action_entries([vue_totale, vue_partielle, ajouter, supprimer, reset]);

    let menubar = 
    {
        let file_menu = 
        {
            let about_menu_item = gio::MenuItem::new(Some("A propos"), Some("app.about"));
            let quit_menu_item = gio::MenuItem::new(Some("Quit"), Some("app.quit"));

            let file_menu = gio::Menu::new();
            file_menu.append_item(&about_menu_item);
            file_menu.append_item(&quit_menu_item);
            file_menu
        };

        let eciv_menu = 
        {
            let vue_totale_menu_item = gio::MenuItem::new(Some("Vue totale"), Some("app.vue_totale"));
            let vue_partielle_menu_item = gio::MenuItem::new(Some("Vue partielle"), Some("app.vue_partielle"));
            let ajouter_menu_item = gio::MenuItem::new(Some("Ajouter"), Some("app.ajouter"));
            let supprimer_menu_item = gio::MenuItem::new(Some("Supprimer"), Some("app.supprimer"));
            let reset_menu_item = gio::MenuItem::new(Some("Réinitialiser"), Some("app.reset"));

            let eciv_menu = gio::Menu::new();
            eciv_menu.append_item(&vue_totale_menu_item);
            eciv_menu.append_item(&vue_partielle_menu_item);
            eciv_menu.append_item(&ajouter_menu_item);
            eciv_menu.append_item(&supprimer_menu_item);
            eciv_menu.append_item(&reset_menu_item);
            eciv_menu
        };

        let menubar = gio::Menu::new();
        menubar.append_submenu(Some("File"), &file_menu);
        menubar.append_submenu(Some("Etat civil"), &eciv_menu);
        menubar
    };

    app.set_menubar(Some(&menubar));
}

fn on_activate(app: &gtk::Application) 
{
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Gestion des individus")
        .default_width(350)
        .default_height(350)
        .show_menubar(true)
        .build();

    window.present();
}

//#[derive(Debug)]
struct Colonne
{
    nom: String,
    prenom: String,
    naissance: String
}

fn vue_totale ()
{
    let url = "mysql://denis:12345678@localhost:3306/rust";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let rows = conn.query_map ( "SELECT lib_nom, lib_prenom, lib_naissance FROM individu, nom, prenom, naissance
                                                  WHERE nom=id_nom AND prenom=id_prenom AND naissance=id_naissance",
                                                | ( nom, prenom, naissance )| Colonne { nom, prenom, naissance } ).unwrap();
// affichage
    let store = affichage();    // création de la grille d'affichage
    for row in rows
    {
        store.append ( &BoxedAnyObject::new ( Row 
        {
            col1: row.nom,
            col2: row.prenom,
            col3: row.naissance
        } ) );
    }
}

fn vue_partielle()
{
    let ( entry_for_button1, entry_for_button2, entry_for_button3, button ) = saisie();
    button.connect_clicked(move |_| 
    {
        // text() renvoie un glib::GString ; to_string() donne un String Rust.
        let nom = entry_for_button1.text().to_string();
        if nom.len() >= 2 && nom.len() <= 30 {
            println!("Texte saisi : {nom}") 
        } 
        else 
        {
            let app: &gtk::Application = &Default::default();
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("Nom invalide."));
            window.set_default_size(400, 60);
            window.present();
            return;
        }
        println!("{nom}");
// requête
        let url = "mysql://denis:12345678@localhost:3306/rust";
        let pool = Pool::new(url).unwrap();
        let mut conn = pool.get_conn().unwrap();
        let query = format!("SELECT lib_nom, lib_prenom, lib_naissance FROM individu, nom, prenom, naissance
                                    WHERE nom=id_nom AND prenom=id_prenom AND naissance=id_naissance AND lib_nom LIKE '{nom}'");
        let rows = conn.query_map ( query, | ( nom, prenom, naissance )|
                                                             Colonne { nom, prenom, naissance } ).unwrap();
// affichage du résultat
        let store = affichage();    // création de la grille d'affichage
        for row in rows
        {
            store.append ( &BoxedAnyObject::new ( Row 
            {
                col1: row.nom,
                col2: row.prenom,
                col3: row.naissance
            } ) );
        }
    });
}

//#[derive(Debug)]
struct Colonne1
{
    nom: u64
}

struct Colonne2
{
    prenom: u64
}

struct Colonne3
{
    naissance: u64
}

fn ajouter_individu()
{
    let ( entry_for_button1, entry_for_button2, entry_for_button3, button ) = saisie();
    button.connect_clicked(move |_| 
    {
        // text() renvoie un glib::GString ; to_string() donne un String Rust.
        let nom = entry_for_button1.text().to_string();
        if nom.len() >= 2 && nom.len() <= 30 
        {
            println!("Texte saisi : {nom}") 
        } 
        else 
        {
            let app: &gtk::Application = &Default::default();
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("Nom invalide."));
            window.set_default_size(400, 60);
            window.present();
            return;
        }
        println!("{nom}");
        let prenom = entry_for_button2.text().to_string();
        if prenom.len() >= 2 && prenom.len() <= 30 
        {
            println!("Texte saisi : {prenom}") 
        } 
        else 
        {
            let app: &gtk::Application = &Default::default();
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("Prénom invalide."));
            window.set_default_size(400, 60);
            window.present();
            return;
        }
        println!("{prenom}");
        let naissance = entry_for_button3.text().to_string();
        if naissance.len() == 4 
        {
            println!("Texte saisi : {naissance}") 
        } 
        else 
        {
            let app: &gtk::Application = &Default::default();
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("Année de naissance invalide."));
            window.set_default_size(400, 60);
            window.present();
            return;
        }
        println!("{naissance}");

        let url = "mysql://denis:12345678@localhost:3306/rust";
        let pool = Pool::new(url).unwrap();
        let mut conn = pool.get_conn().unwrap();
        let requete = String::from ( format! ( "SELECT lib_nom, lib_prenom, lib_naissance FROM individu, nom, prenom, naissance WHERE nom=id_nom
                                                 AND prenom=id_prenom AND naissance=id_naissance AND lib_nom LIKE '{nom}'
                                                 AND lib_prenom LIKE '{prenom}' AND lib_naissance LIKE '{naissance}'" ) );
        let a1 = nom.clone();
        let a2 = prenom.clone();
        let a3 = naissance.clone();
        let rows = conn.query_map ( requete,  | ( nom, prenom, naissance )|
                                                                 Colonne { nom, prenom, naissance } ).unwrap();
        if rows.len() > 0
        {
            let app: &gtk::Application = &Default::default();
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("L'état civil saisi est déjà dans la base de données."));
            window.set_default_size(400, 60);
            window.present();
		}
		else				// insertion
		{
            let mut commit = true;
            let mut tx = conn.start_transaction (TxOpts::default()).unwrap();
            // traitement du nom
            let requete = String::from ( format! ( "SELECT id_nom FROM nom WHERE lib_nom LIKE '{nom}'" ) );
            let nom = a1.clone();
            let mut conn = pool.get_conn().unwrap();
            let rows = conn.query_map ( requete,  | ( nom, ) | Colonne1 { nom } ).unwrap();
            let id_nom: u64;
            if rows.len() > 0
            {
                id_nom= rows[0].nom;
            }
            else
            {
                tx.exec_drop ( "INSERT INTO nom ( lib_nom ) VALUES (?)", (&nom, ) ).unwrap();
                id_nom = tx.last_insert_id().unwrap();
                if id_nom == 0
                {
                    commit = false
                }
            }
            println!("id_nom = {id_nom}");
            // traitement du prénom
            let requete = String::from ( format! ( "SELECT id_prenom FROM prenom WHERE lib_prenom LIKE '{prenom}'" ) );
            let prenom = a2.clone();
            let mut conn = pool.get_conn().unwrap();
            let rows = conn.query_map ( requete,  | ( prenom, ) | Colonne2 { prenom } ).unwrap();
            let id_prenom: u64;
            if rows.len() > 0
            {
                id_prenom= rows[0].prenom;
            }
            else
            {
                tx.exec_drop ( "INSERT INTO prenom ( lib_prenom ) VALUES (?)", (&prenom, ) ).unwrap();
                id_prenom = tx.last_insert_id().unwrap();
                if id_prenom == 0
                {
                    commit = false
                }
            }
            println!("id_prenom = {id_prenom}");
            // traitement de l'année de naissance
            let requete = String::from ( format! ( "SELECT id_naissance FROM naissance WHERE lib_naissance LIKE '{naissance}'" ) );
            let naissance = a3.clone();
            let mut conn = pool.get_conn().unwrap();
            let rows = conn.query_map ( requete,  | ( naissance, ) | Colonne3 { naissance } ).unwrap();
            let id_naissance: u64;
            if rows.len() > 0
            {
                id_naissance = rows[0].naissance;
            }
            else
            {
                tx.exec_drop ( "INSERT INTO naissance ( lib_naissance ) VALUES (?)", (&naissance, ) ).unwrap();
                id_naissance = tx.last_insert_id().unwrap();
                if id_naissance == 0
                {
                    commit = false
                }
            }
            println!("id_naissance = {id_naissance}");
         
            // update table individu
            tx.exec_drop ( "INSERT INTO individu (nom, prenom, naissance) VALUES ( ?, ?, ? )", ( id_nom, id_prenom, id_naissance ) ).unwrap();
            // clore la transaction
            if commit
            {
                tx.commit().unwrap();
            }
            else
            {
                tx.rollback().unwrap();
            }
        } 
    });
}

fn supprimer_individu()
{
    let ( entry_for_button1, entry_for_button2, entry_for_button3, button ) = saisie();
    button.connect_clicked(move |_| 
    {
        // text() renvoie un glib::GString ; to_string() donne un String Rust.
        let nom = entry_for_button1.text().to_string();
        if nom.len() >= 2 && nom.len() <= 30 
        {
            println!("Texte saisi : {nom}") 
        } 
        else 
        {
            let app: &gtk::Application = &Default::default();
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("Nom invalide."));
            window.set_default_size(400, 60);
            window.present();
            return;
        }
        println!("{nom}");
        let prenom = entry_for_button2.text().to_string();
        if prenom.len() >= 2 && prenom.len() <= 30 
        {
            println!("Texte saisi : {prenom}") 
        } 
        else 
        {
            let app: &gtk::Application = &Default::default();
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("Prénom invalide."));
            window.set_default_size(400, 60);
            window.present();
            return;
        }
        println!("{prenom}");
        let naissance = entry_for_button3.text().to_string();
        if naissance.len() == 4 
        {
            println!("Texte saisi : {naissance}") 
        } 
        else 
        {
            let app: &gtk::Application = &Default::default();
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("Année de naissance invalide."));
            window.set_default_size(400, 60);
            window.present();
            return;
        }
        println!("{naissance}");

        let url = "mysql://denis:12345678@localhost:3306/rust";
        let pool = Pool::new(url).unwrap();
        let mut conn = pool.get_conn().unwrap();
        let requete = String::from ( format! ( "SELECT lib_nom, lib_prenom, lib_naissance FROM individu, nom, prenom, naissance WHERE nom=id_nom
                                                 AND prenom=id_prenom AND naissance=id_naissance AND lib_nom LIKE '{nom}'
                                                 AND lib_prenom LIKE '{prenom}' AND lib_naissance LIKE '{naissance}'" ) );
        let a1 = nom.clone();
        let a2 = prenom.clone();
        let a3 = naissance.clone();
        let rows = conn.query_map ( requete,  | ( nom, prenom, naissance )|
                                                                 Colonne { nom, prenom, naissance } ).unwrap();
        if rows.len() == 0
        {
            let app: &gtk::Application = &Default::default();
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("L'état civil saisi n'existe pas dans la base de données."));
            window.set_default_size(400, 60);
            window.present();
            return;
		}
		else				// suppression
		{
            let nom = a1.clone();
            let prenom = a2.clone();
            let naissance = a3.clone();
            conn.exec_drop ( "DELETE FROM individu WHERE nom IN (SELECT id_nom FROM nom WHERE lib_nom LIKE ?) AND 
                                prenom IN (SELECT id_prenom FROM prenom WHERE lib_prenom LIKE ?) AND 
                                naissance IN (SELECT id_naissance FROM naissance WHERE lib_naissance LIKE ?)",
                                ( &nom, &prenom, &naissance ) ).unwrap();
        }
    });
}

fn reset_database ()
{
    let url = "mysql://denis:12345678@localhost:3306/rust";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    conn.query_drop ( r"DROP TABLE IF EXISTS individu;
                        CREATE TABLE individu ( nom INT NOT NULL, prenom INT NOT NULL, naissance INT NOT NULL )" ).unwrap();
    conn.query_drop ( r"DROP TABLE IF EXISTS naissance;
                        CREATE TABLE naissance ( id_naissance INT NOT NULL AUTO_INCREMENT, lib_naissance CHAR(4) NOT NULL, PRIMARY KEY ( id_naissance ) )" ).unwrap();
    conn.query_drop ( r"DROP TABLE IF EXISTS nom;
                        CREATE TABLE nom ( id_nom INT NOT NULL AUTO_INCREMENT, lib_nom VARCHAR(30) DEFAULT NULL, PRIMARY KEY ( id_nom ) )" ).unwrap();
    conn.query_drop ( r"DROP TABLE IF EXISTS prenom;
                        CREATE TABLE prenom ( id_prenom INT NOT NULL AUTO_INCREMENT, lib_prenom VARCHAR(30) DEFAULT NULL, PRIMARY KEY ( id_prenom ) )" ).unwrap();
}

fn affichage() -> gio::ListStore
{
    let app: &gtk::Application = &Default::default();
    let window = gtk::ApplicationWindow::builder()
        .default_width(320)
        .default_height(480)
        .application(app)
        .title("Résultats")
        .build();

    let store = gio::ListStore::new::<BoxedAnyObject>();
    let store2 = store.clone();
    let sel = gtk::SingleSelection::new(Some(store2));  // store2 absorbé, il reste store
    let columnview = gtk::ColumnView::new(Some(sel));

    let col1factory = gtk::SignalListItemFactory::new();
    let col2factory = gtk::SignalListItemFactory::new();
    let col3factory = gtk::SignalListItemFactory::new();

    col1factory.connect_setup(move |_factory, item| 
    {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = GridCell::default();
        item.set_child(Some(&row));
    });

    col1factory.connect_bind(move |_factory, item| 
    {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let child = item.child().and_downcast::<GridCell>().unwrap();
        let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let r: Ref<Row> = entry.borrow();
        let ent = Entry2 
        {
            name: r.col1.to_string(),
        };
        child.set_entry(&ent);
    });

    col2factory.connect_setup(move |_factory, item| 
    {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = GridCell::default();
        item.set_child(Some(&row));
    });

    col2factory.connect_bind(move |_factory, item| 
    {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let child = item.child().and_downcast::<GridCell>().unwrap();
        let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let r: Ref<Row> = entry.borrow();
        let ent = Entry2 
        {
            name: r.col2.to_string(),
        };
        child.set_entry(&ent);
    });

    col3factory.connect_setup(move |_factory, item| 
    {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = GridCell::default();
        item.set_child(Some(&row));
    });

    col3factory.connect_bind(move |_factory, item| 
    {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let child = item.child().and_downcast::<GridCell>().unwrap();
        let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let r: Ref<Row> = entry.borrow();
        let ent = Entry2 
        {
            name: r.col3.to_string(),
        };
        child.set_entry(&ent);
    });
    
    let col1 = gtk::ColumnViewColumn::new(Some("Nom                      "), Some(col1factory));
    let col2 = gtk::ColumnViewColumn::new(Some("Prénom                   "), Some(col2factory));
    let col3 = gtk::ColumnViewColumn::new(Some("Année de naissance"), Some(col3factory));
    
    columnview.append_column(&col1);
    columnview.append_column(&col2);
    columnview.append_column(&col3);

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never) // Disable horizontal scrolling
        .build();

    scrolled_window.set_child(Some(&columnview));

    window.set_child(Some(&scrolled_window));
    window.present();
    store
}

fn saisie() -> ( Entry, Entry, Entry, Button )
{
        let app: &gtk::Application = &Default::default();
    let entry1 = Entry::builder()
        .placeholder_text("Nom…")
        .hexpand(true)
        .build();

    let entry2 = Entry::builder()
        .placeholder_text("Prénom…")
        .hexpand(true)
        .build();

    let entry3 = Entry::builder()
        .placeholder_text("Année de naissance…")
        .hexpand(true)
        .build();

    let button = Button::with_label("Valider");

    // Clone le widget pour pouvoir l'utiliser dans la closure ci-dessous.
    let entry_for_button1 = entry1.clone();
    let entry_for_button2 = entry2.clone();
    let entry_for_button3 = entry3.clone();

    let layout = gtk::Box::new(Orientation::Vertical, 8);
    layout.set_margin_top(12);
    layout.set_margin_bottom(12);
    layout.set_margin_start(12);
    layout.set_margin_end(12);
    layout.append(&entry1);
    layout.append(&entry2);
    layout.append(&entry3);
    layout.append(&button);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Recherche ou saisie")
        .default_width(320)
        .child(&layout)
        .build();

    window.present();
    ( entry_for_button1, entry_for_button2, entry_for_button3, button )
}