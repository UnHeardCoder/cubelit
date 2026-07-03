import { describe, it, expect } from "vitest";
import { getField, setField, asBool, parseIniValue, upsertIniValue } from "./parsers";

describe("properties format", () => {
  const sample = "difficulty=normal\n# comment\nmax-players=20\n";

  it("reads a key", () => {
    expect(getField(sample, "properties", "difficulty")).toBe("normal");
    expect(getField(sample, "properties", "max-players")).toBe("20");
  });

  it("ignores comments and missing keys", () => {
    expect(getField(sample, "properties", "comment")).toBeNull();
    expect(getField(sample, "properties", "nope")).toBeNull();
  });

  it("updates an existing key in place", () => {
    const out = setField(sample, "properties", "difficulty", "hard");
    expect(getField(out, "properties", "difficulty")).toBe("hard");
    // Other keys untouched.
    expect(getField(out, "properties", "max-players")).toBe("20");
  });

  it("appends a new key", () => {
    const out = setField(sample, "properties", "motd", "Hello");
    expect(getField(out, "properties", "motd")).toBe("Hello");
  });
});

describe("ini format", () => {
  const sample = "[ServerSettings]\nXPMultiplier=1\nServerPVE=False\n\n[SessionSettings]\nSessionName=Old\n";

  it("reads sectioned keys", () => {
    expect(getField(sample, "ini", "XPMultiplier", "[ServerSettings]")).toBe("1");
    expect(getField(sample, "ini", "SessionName", "[SessionSettings]")).toBe("Old");
  });

  it("does not cross section boundaries", () => {
    expect(getField(sample, "ini", "SessionName", "[ServerSettings]")).toBeNull();
  });

  it("updates within the right section", () => {
    const out = setField(sample, "ini", "XPMultiplier", "5", "[ServerSettings]");
    expect(getField(out, "ini", "XPMultiplier", "[ServerSettings]")).toBe("5");
    expect(getField(out, "ini", "SessionName", "[SessionSettings]")).toBe("Old");
  });

  it("creates a missing section when upserting", () => {
    const out = upsertIniValue("", "[NewSection]", "Key", "Val");
    expect(parseIniValue(out, "[NewSection]", "Key")).toBe("Val");
  });

  it("asBool coerces correctly", () => {
    expect(asBool(getField(sample, "ini", "ServerPVE", "[ServerSettings]"))).toBe(false);
    const on = setField(sample, "ini", "ServerPVE", "True", "[ServerSettings]");
    expect(asBool(getField(on, "ini", "ServerPVE", "[ServerSettings]"))).toBe(true);
  });
});
