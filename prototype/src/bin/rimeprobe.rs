use rime_api::{
    DeployResult, KeyEvent, Traits, create_session, full_deploy_and_wait, initialize, setup,
};

fn main() {
    let mut traits = Traits::new();
    traits
        .set_shared_data_dir("/usr/share/rime-data")
        .set_user_data_dir("/tmp/rime-user")
        .set_distribution_name("rimeprobe")
        .set_distribution_code_name("rimeprobe")
        .set_distribution_version("0.0.1")
        .set_log_dir("/tmp/rime-user/log")
        .set_min_log_level(2);

    setup(&mut traits);
    initialize(&mut traits);
    rime_api::set_notification_handler(|kind, value| {
        println!("NOTIFY {kind}: {value}");
    });

    println!("deploying wubi86...");
    let deploy = full_deploy_and_wait();
    println!("deploy = {deploy:?}");
    if !matches!(deploy, DeployResult::Success) {
        return;
    }

    let mut session = create_session().expect("create session");
    session.select_schema("wubi86").expect("select wubi86");

    for seq in ["a", "wqiy", "wq"] {
        println!("--- sequence {seq:?}");
        session.simulate_key_sequence(seq).expect("keys");

        if let Some(ctx) = session.context() {
            let composition = ctx.composition();
            println!("preedit = {:?}", composition.preedit);
            let menu = ctx.menu();
            println!("candidates = {}", menu.num_candidates);
            for (i, candidate) in menu.candidates.iter().enumerate() {
                println!("  {i}: {} {:?}", candidate.text, candidate.comment);
            }
        } else {
            println!("no context");
        }

        let status = session.process_key(KeyEvent::new(0x20, 0));
        println!("space -> {status:?}");
        if let Some(commit) = session.commit() {
            println!("commit = {:?}", commit.text());
        }

        session.process_key(KeyEvent::new(0xff1b, 0));
    }

    session.close().expect("close session");

    let mut pinyin = create_session().expect("create pinyin session");
    pinyin
        .select_schema("pinyin_simp")
        .expect("select pinyin_simp");
    for seq in ["nihao", "wqiy"] {
        println!("--- pinyin sequence {seq:?}");
        pinyin.simulate_key_sequence(seq).expect("pinyin keys");
        if let Some(ctx) = pinyin.context() {
            let composition = ctx.composition();
            println!("preedit = {:?}", composition.preedit);
            let menu = ctx.menu();
            println!("candidates = {}", menu.num_candidates);
            for (i, candidate) in menu.candidates.iter().enumerate() {
                println!("  {i}: {} {:?}", candidate.text, candidate.comment);
            }
        }
        pinyin.process_key(KeyEvent::new(0x20, 0));
        if let Some(commit) = pinyin.commit() {
            println!("commit = {:?}", commit.text());
        }
        pinyin.process_key(KeyEvent::new(0xff1b, 0));
    }
    pinyin.close().expect("close pinyin session");

    let mut punct = create_session().expect("create punctuation session");
    punct.select_schema("wubi86").expect("select wubi86");
    for seq in [".", ",", "?"] {
        println!("--- punctuation {seq:?}");
        punct.simulate_key_sequence(seq).expect("punct keys");
        if let Some(ctx) = punct.context() {
            let composition = ctx.composition();
            println!("preedit = {:?}", composition.preedit);
            let menu = ctx.menu();
            println!("candidates = {}", menu.num_candidates);
            for (i, candidate) in menu.candidates.iter().enumerate() {
                println!("  {i}: {} {:?}", candidate.text, candidate.comment);
            }
        }
        if let Some(commit) = punct.commit() {
            println!("commit = {:?}", commit.text());
        }
        punct.process_key(KeyEvent::new(0xff1b, 0));
    }
    punct.close().expect("close punctuation session");

    rime_api::finalize();
}
