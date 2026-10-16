//! How-to guides for every exercise and daily routine, shown on the Guides
//! page. General desk-health guidance, not medical advice.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideGroup {
    Exercise,
    Routine,
}

/// What the guide's "log it" button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideAction {
    Plank,
    Stretching,
    Walk,
    Water,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideColor {
    Orange,
    Purple,
    Teal,
    Indigo,
    Cyan,
    Blue,
    Pink,
    Yellow,
}

pub struct Guide {
    pub id: &'static str,
    pub group: GuideGroup,
    pub title: &'static str,
    pub symbol: &'static str,
    pub color: GuideColor,
    /// One line under the title.
    pub summary: &'static str,
    pub why: &'static str,
    pub steps: &'static [&'static str],
    pub how_long: &'static str,
    pub tips: &'static [&'static str],
    pub action: Option<(GuideAction, &'static str)>,
}

pub const GUIDES: &[Guide] = &[
    Guide {
        id: "plank",
        group: GuideGroup::Exercise,
        title: "Plank",
        symbol: "figure.core.training",
        color: GuideColor::Orange,
        summary: "Hold a straight line to build the core muscles that support your back.",
        why: "A strong core takes load off your lower back, which helps after long hours sitting. \
              The plank works your whole midsection with no equipment.",
        steps: &[
            "Lie face down, then rest on your forearms with elbows directly under your shoulders.",
            "Stretch your legs back and lift onto your toes, feet about hip-width apart.",
            "Make one straight line from head to heels: hips level, not sagging or raised.",
            "Tighten your stomach as if bracing for a light punch and squeeze your glutes.",
            "Look at the floor just ahead of your hands so your neck stays neutral.",
            "Breathe slowly and steadily while you hold. Lower your knees to finish.",
        ],
        how_long: "Hold 20–30 seconds to start, work up to 60 seconds. Do 2–3 rounds with a short rest.",
        tips: &[
            "Too hard? Keep your knees on the floor and hold the same straight line from head to knees.",
            "Don't hold your breath, and stop when your hips start to drop. Good form beats a long hold.",
            "Stop if you feel pain in your lower back or shoulders.",
        ],
        action: Some((GuideAction::Plank, "Log a Plank")),
    },
    Guide {
        id: "stretching",
        group: GuideGroup::Exercise,
        title: "Stretching",
        symbol: "figure.flexibility",
        color: GuideColor::Purple,
        summary: "A 5-minute desk stretch routine for neck, shoulders, back, wrists and hips.",
        why: "Sitting still shortens and stiffens muscles. A few gentle stretches every hour relieve \
              neck and shoulder tension, open your chest and get blood moving again.",
        steps: &[
            "Neck: sit tall, tilt your right ear toward your right shoulder. Hold, then switch sides.",
            "Shoulders: roll both shoulders up, back and down slowly, 10 times.",
            "Chest: clasp your hands behind your back, straighten your arms and gently lift your chest.",
            "Upper back: hold your arms out in front, round your back and push your hands away.",
            "Spine: sit tall and twist to the right, holding the chair back. Hold, then twist left.",
            "Wrists: arm straight out, palm up, gently pull your fingers back with the other hand. Switch.",
            "Hips: stand, hold the desk, bend one knee and pull that foot toward your glutes. Switch.",
        ],
        how_long: "Hold each stretch 15–30 seconds. The whole routine takes about 5 minutes.",
        tips: &[
            "Stretch to mild tension, never pain, and don't bounce.",
            "Breathe out as you go into each stretch.",
            "Do it once an hour, for example after each break notice.",
        ],
        action: Some((GuideAction::Stretching, "Log Stretching")),
    },
    Guide {
        id: "walk",
        group: GuideGroup::Exercise,
        title: "Walk",
        symbol: "figure.walk",
        color: GuideColor::Teal,
        summary: "A brisk 10-minute walk to wake up your body and mind.",
        why: "Walking is the easiest way to reach your daily movement goal. It helps circulation, \
              blood sugar and mood, and often clears your head when you're stuck on a problem.",
        steps: &[
            "Stand up and leave your desk; outdoors is best if you can.",
            "Walk tall: head up, shoulders relaxed and down, eyes forward.",
            "Swing your arms naturally with elbows slightly bent.",
            "Pick a brisk pace: you can talk, but singing would be hard.",
            "Keep going for about 10 minutes, then slow down for the last minute.",
        ],
        how_long: "10 minutes counts as one walk. Three walks reach the 30-minute daily goal.",
        tips: &[
            "Take the stairs, or take phone calls and meetings on foot.",
            "Leave your laptop behind so it's a real break.",
        ],
        action: Some((GuideAction::Walk, "Log a Walk")),
    },
    Guide {
        id: "breathing",
        group: GuideGroup::Exercise,
        title: "Deep Breathing",
        symbol: "wind",
        color: GuideColor::Indigo,
        summary: "Slow 4-4-6 breathing to calm down and refocus.",
        why: "Slow breathing with a longer exhale calms your nervous system. It's a quick reset \
              when you feel stressed or scattered.",
        steps: &[
            "Sit tall with both feet on the floor and relax your shoulders.",
            "Put one hand on your belly. Breathe in through your nose for 4 seconds so your belly rises.",
            "Hold your breath gently for 4 seconds.",
            "Breathe out slowly through your mouth for 6 seconds.",
            "Repeat 5 to 10 times.",
        ],
        how_long: "1–2 minutes. The Break Timer has a 5-minute Breathe mode.",
        tips: &[
            "If holding feels uncomfortable, skip the hold and just breathe in 4, out 6.",
            "Closing your eyes helps you focus on the count.",
        ],
        action: None,
    },
    Guide {
        id: "eye_break",
        group: GuideGroup::Exercise,
        title: "20-20-20 Eye Break",
        symbol: "eye.fill",
        color: GuideColor::Cyan,
        summary: "Every 20 minutes, look 20 feet away for 20 seconds.",
        why: "Focusing on a screen up close keeps your eye muscles tense and makes you blink less. \
              Looking far away relaxes them and reduces dry, tired eyes.",
        steps: &[
            "Every 20 minutes, look away from the screen.",
            "Focus on something about 20 feet (6 meters) away, like a window or the far end of the room.",
            "Keep looking for at least 20 seconds.",
            "Blink fully and slowly a few times before going back to work.",
        ],
        how_long: "20 seconds, about 6 times a day. The Break Timer has a 20-20-20 mode.",
        tips: &[
            "Keep the screen about an arm's length away, with the top at or just below eye level.",
            "Reduce glare: match screen brightness to the room and avoid a bright window behind it.",
        ],
        action: None,
    },
    Guide {
        id: "posture",
        group: GuideGroup::Exercise,
        title: "Posture Check",
        symbol: "figure.stand",
        color: GuideColor::Purple,
        summary: "A 30-second check from feet to head.",
        why: "Small posture habits add up over a workday. A quick check lowers strain on your neck, \
              shoulders, wrists and lower back.",
        steps: &[
            "Feet flat on the floor, knees at about 90°.",
            "Sit all the way back so the chair supports your lower back.",
            "Relax your shoulders down; elbows close to your body, bent at about 90°.",
            "Keep wrists straight on the keyboard and mouse, not bent up or down.",
            "Head over your shoulders, not pushed forward; chin slightly tucked.",
            "Screen an arm's length away, top of the screen at eye level.",
        ],
        how_long: "30 seconds, about 4 times a day.",
        tips: &[
            "Adjust the chair and desk once, properly; then the check is just a quick reminder.",
            "Using a laptop all day? A stand plus an external keyboard makes good posture much easier.",
        ],
        action: None,
    },
    Guide {
        id: "hydration",
        group: GuideGroup::Routine,
        title: "Hydration",
        symbol: "drop.fill",
        color: GuideColor::Blue,
        summary: "Drink about 2 liters across the day, a little at a time.",
        why: "Even mild dehydration can cause headaches, tiredness and poor focus. Steady sips work \
              better than drinking a lot at once.",
        steps: &[
            "Keep a glass or bottle on your desk where you can see it.",
            "Drink a glass when you start work and one with each meal.",
            "Have a few sips at every break notice.",
            "Log each glass (250 ml) or bottle (500 ml) in the app or the Control Center.",
        ],
        how_long: "All day. 8 glasses of 250 ml reach the 2-liter goal.",
        tips: &[
            "Drink more when it's hot or after exercise.",
            "Pale yellow urine usually means you're drinking enough.",
            "Coffee and tea count toward fluids, but water is best.",
        ],
        action: Some((GuideAction::Water, "Log a Glass (250 ml)")),
    },
    Guide {
        id: "movement",
        group: GuideGroup::Routine,
        title: "Daily Movement",
        symbol: "figure.run",
        color: GuideColor::Orange,
        summary: "30 minutes of moderate activity a day, in any chunks you like.",
        why: "Regular movement is one of the best things you can do for your heart, mood and energy, \
              and it offsets long hours of sitting.",
        steps: &[
            "Pick activities that raise your heart rate a bit: brisk walking, cycling, stairs, a workout.",
            "Split it up if that's easier: three 10-minute walks count just as well.",
            "Log a Walk (+10 min) or a Plank (+1 min) from the Control Center as you go.",
            "Use the Walk and Plank guides for technique.",
        ],
        how_long: "30 minutes a day in total.",
        tips: &[
            "Schedule it like a meeting: a walk right after lunch is easy to keep up.",
            "Any movement beats none. Start small and build up.",
        ],
        action: Some((GuideAction::Walk, "Log a Walk (+10 min)")),
    },
    Guide {
        id: "screen_free_meal",
        group: GuideGroup::Routine,
        title: "Screen-Free Meal",
        symbol: "fork.knife",
        color: GuideColor::Pink,
        summary: "Eat one meal a day away from every screen.",
        why: "Eating at the desk means no real break for your eyes or mind, and you tend to eat faster \
              and notice less when you're full.",
        steps: &[
            "Step away from your desk to eat; another room or outside is best.",
            "Leave your phone in your pocket or face down.",
            "Take about 20 minutes and eat slowly.",
            "Tick it off in Daily Routines when you're done.",
        ],
        how_long: "About 20 minutes, once a day.",
        tips: &["Eating with a colleague or friend makes it easier to skip the screen."],
        action: None,
    },
    Guide {
        id: "vitamins",
        group: GuideGroup::Routine,
        title: "Vitamins & Supplements",
        symbol: "pills.fill",
        color: GuideColor::Yellow,
        summary: "Take your daily vitamins or prescribed supplements at the same time each day.",
        why: "Supplements only help when taken regularly. Linking them to a fixed moment makes them \
              hard to forget.",
        steps: &[
            "Take only what your doctor or pharmacist recommends.",
            "Pick a fixed moment, such as breakfast; many vitamins absorb better with food.",
            "Keep them where you'll see them at that moment.",
            "Tick it off in Daily Routines after taking them.",
        ],
        how_long: "Once a day.",
        tips: &[
            "Don't take more than the stated dose, and check with a pharmacist before combining products.",
        ],
        action: None,
    },
];

pub fn by_id(id: &str) -> Option<&'static Guide> {
    GUIDES.iter().find(|g| g.id == id)
}

/// Guide for a daily-routine task id (see `DailyRecord::with_date`).
pub fn for_routine(task_id: &str) -> Option<&'static Guide> {
    let id = match task_id {
        "water" => "hydration",
        "eye_breaks" => "eye_break",
        "stand_stretch" => "stretching",
        "posture" => "posture",
        "exercise" => "movement",
        "mindful_lunch" => "screen_free_meal",
        "vitamins" => "vitamins",
        _ => return None,
    };
    by_id(id)
}
