import type { SubtitleLine } from "./types";

const baseTime = Date.now();

export const MOCK_SUBTITLES: SubtitleLine[] = [
	{ id: "mock-01", text: "Good evening everyone.", timestamp: baseTime + 0 },
	{
		id: "mock-02",
		text: "Welcome to this live caption preview.",
		timestamp: baseTime + 120,
	},
	{
		id: "mock-03",
		text: "We are checking subtitle readability first.",
		timestamp: baseTime + 240,
	},
	{ id: "mock-04", text: "Short line.", timestamp: baseTime + 360 },
	{
		id: "mock-05",
		text: "This sentence is intentionally much longer so we can inspect wrapping behavior on smaller windows and narrow aspect ratios.",
		timestamp: baseTime + 480,
	},
	{
		id: "mock-06",
		text: "Please keep your microphone muted for now.",
		timestamp: baseTime + 600,
	},
	{
		id: "mock-07",
		text: "The speaker will begin in thirty seconds.",
		timestamp: baseTime + 720,
	},
	{ id: "mock-08", text: "Another short one.", timestamp: baseTime + 840 },
	{
		id: "mock-09",
		text: "We can also validate vertical spacing between source and translation lines.",
		timestamp: baseTime + 960,
	},
	{
		id: "mock-10",
		text: "Animation timing should feel calm, not distracting.",
		timestamp: baseTime + 1080,
	},
	{
		id: "mock-11",
		text: "Can everyone at the back read this clearly?",
		timestamp: baseTime + 1200,
	},
	{
		id: "mock-12",
		text: "Contrast is critical for projected environments.",
		timestamp: baseTime + 1320,
	},
	{
		id: "mock-13",
		text: "Now we test a number: 1234567890.",
		timestamp: baseTime + 1440,
	},
	{
		id: "mock-14",
		text: "Now we test punctuation: commas, periods, and question marks?",
		timestamp: baseTime + 1560,
	},
	{
		id: "mock-15",
		text: "Layout should still look stable with mixed-length content.",
		timestamp: baseTime + 1680,
	},
	{
		id: "mock-16",
		text: "Please adjust subtitle position to about sixty percent.",
		timestamp: baseTime + 1800,
	},
	{
		id: "mock-17",
		text: "This helps evaluate readability over presentation slides.",
		timestamp: baseTime + 1920,
	},
	{
		id: "mock-18",
		text: "If the text is too large, reduce font size slightly.",
		timestamp: baseTime + 2040,
	},
	{
		id: "mock-19",
		text: "If the text is too dim, increase opacity.",
		timestamp: baseTime + 2160,
	},
	{
		id: "mock-20",
		text: "A quick reminder: this is mock mode only.",
		timestamp: baseTime + 2280,
	},
	{
		id: "mock-21",
		text: "No real audio is being processed right now.",
		timestamp: baseTime + 2400,
	},
	{
		id: "mock-22",
		text: "This line is here to ensure we exceed twenty rows.",
		timestamp: baseTime + 2520,
	},
	{
		id: "mock-23",
		text: "Final visual check before implementation work starts.",
		timestamp: baseTime + 2640,
	},
	{
		id: "mock-24",
		text: "Press Stop to leave mock mode and return to idle.",
		timestamp: baseTime + 2760,
	},
];

export const MOCK_TRANSLATIONS: Record<string, string> = {
	"mock-01": "สวัสดีตอนเย็นทุกคน",
	"mock-02": "ยินดีต้อนรับสู่การพรีวิวคำบรรยายสด",
	"mock-03": "เรากำลังตรวจสอบความอ่านง่ายของคำบรรยายก่อน",
	"mock-04": "บรรทัดสั้น",
	"mock-05": "ประโยคนี้ตั้งใจทำให้ยาวเป็นพิเศษเพื่อทดสอบการตัดบรรทัดบนหน้าต่างขนาดเล็กและอัตราส่วนจอที่แคบ",
	"mock-06": "กรุณาปิดไมโครโฟนไว้ก่อนตอนนี้",
	"mock-07": "ผู้พูดจะเริ่มภายในสามสิบวินาที",
	"mock-08": "อีกบรรทัดสั้นๆ",
	"mock-09": "เรายังสามารถตรวจระยะห่างแนวตั้งระหว่างต้นฉบับและคำแปลได้",
	"mock-10": "จังหวะแอนิเมชันควรรู้สึกนิ่งและไม่รบกวน",
	"mock-11": "คนที่อยู่ด้านหลังอ่านได้ชัดไหม",
	"mock-12": "ความเปรียบต่างมีความสำคัญมากเมื่อฉายบนโปรเจ็กเตอร์",
	"mock-13": "ตอนนี้ทดสอบตัวเลข: 1234567890",
	"mock-14": "ตอนนี้ทดสอบเครื่องหมายวรรคตอน: จุลภาค จุด และคำถาม?",
	"mock-15": "เลย์เอาต์ควรยังคงเสถียรแม้ข้อความยาวสั้นปะปนกัน",
	"mock-16": "ลองปรับตำแหน่งคำบรรยายไปประมาณหกสิบเปอร์เซ็นต์",
	"mock-17": "วิธีนี้ช่วยประเมินความอ่านง่ายเมื่อทับบนสไลด์นำเสนอ",
	"mock-18": "ถ้าตัวอักษรใหญ่เกินไปให้ลดขนาดฟอนต์ลงเล็กน้อย",
	"mock-19": "ถ้าตัวอักษรจางเกินไปให้เพิ่มความทึบ",
	"mock-20": "ย้ำอีกครั้ง: ตอนนี้เป็นโหมด mock เท่านั้น",
	"mock-21": "ขณะนี้ยังไม่มีการประมวลผลเสียงจริง",
	"mock-22": "บรรทัดนี้มีไว้เพื่อให้เกินยี่สิบบรรทัดตามต้องการ",
	"mock-23": "ตรวจภาพรวมสุดท้ายก่อนเริ่มพัฒนาฟีเจอร์จริง",
	"mock-24": "กด Stop เพื่อออกจากโหมด mock และกลับหน้าเริ่มต้น",
};

export const MOCK_TRANSLATIONS_2: Record<string, string> = {
	"mock-01": "Good evening everyone.",
	"mock-02": "Welcome to this live caption preview.",
	"mock-03": "We are checking subtitle readability first.",
	"mock-04": "Short line.",
	"mock-05":
		"This sentence is intentionally much longer so we can inspect wrapping behavior on smaller windows and narrow aspect ratios.",
	"mock-06": "Please keep your microphone muted for now.",
	"mock-07": "The speaker will begin in thirty seconds.",
	"mock-08": "Another short one.",
	"mock-09": "We can also validate vertical spacing between source and translation lines.",
	"mock-10": "Animation timing should feel calm, not distracting.",
	"mock-11": "Can everyone at the back read this clearly?",
	"mock-12": "Contrast is critical for projected environments.",
	"mock-13": "Now we test a number: 1234567890.",
	"mock-14": "Now we test punctuation: commas, periods, and question marks?",
	"mock-15": "Layout should still look stable with mixed-length content.",
	"mock-16": "Please adjust subtitle position to about sixty percent.",
	"mock-17": "This helps evaluate readability over presentation slides.",
	"mock-18": "If the text is too large, reduce font size slightly.",
	"mock-19": "If the text is too dim, increase opacity.",
	"mock-20": "A quick reminder: this is mock mode only.",
	"mock-21": "No real audio is being processed right now.",
	"mock-22": "This line is here to ensure we exceed twenty rows.",
	"mock-23": "Final visual check before implementation work starts.",
	"mock-24": "Press Stop to leave mock mode and return to idle.",
};
