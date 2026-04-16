type SpeechRule = (text: string) => string;

function spellDigits(value: string): string {
  return value.split('').join(' ');
}

const expandMmsiDigits: SpeechRule = (text) => {
  return text.replace(/\bMMSI\s*[:=]?\s*(\d{6,12})\b/gi, (_, digits: string) => {
    return `MMSI ${spellDigits(digits)}`;
  });
};

const speechRules: SpeechRule[] = [expandMmsiDigits];

export function formatSpeechText(text: string): string {
  return speechRules.reduce((current, rule) => rule(current), text);
}
