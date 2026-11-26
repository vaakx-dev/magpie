"use strict"

const DEFAULT_SECTION = "general"
const TYPING_PAUSE_MS = 400
const SEGMENTED_LIMIT = 4

const TRACKING_DESCRIPTIONS = {
  Basic: "TrackingBasic",
  Balanced: "TrackingBalanced",
  Strict: "TrackingStrict",
}

const CLEARING_VIEWS = {
  idle: { title: "ClearBrowsingData", description: "ClearBrowsingDataHint", tone: "" },
  running: { title: "Clearing", description: "ClearBrowsingDataHint", tone: "" },
  cleared: { title: "ClearBrowsingData", description: "Cleared", tone: "success" },
  failed: { title: "ClearBrowsingData", description: "ClearFailed", tone: "error" },
}

const DARK_SITE_MODES = { off: "NeverDark", forced: "AlwaysDark" }

const LOCALIZED = "[data-text], [data-placeholder], [data-label]"

const UPDATE_TONES = {
  up_to_date: "success",
  available: "success",
  failed: "error",
  download_failed: "error",
  install_failed: "error",
}
const INSTALLABLE_UPDATES = ["available", "download_failed"]
const BUSY_UPDATES = ["checking", "downloading", "installing"]
const NIGHTLY_VERSION = /^(.+)-nightly\.(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})$/

const VAULT_TITLES = {
  working: "Working",
  unavailable: "Unavailable",
  signed_out: "SignInToBitwarden",
  challenged: "TwoStepLogin",
  locked: "VaultLocked",
  unlocked: "Unlocked",
}
const SIGNED_IN = ["locked", "unlocked"]
const EMAILED_METHODS = ["email", "new_device"]

const SHORTCUTS = [
  { action: "Command Bar", detail: "Open, search and run commands", keys: [["Ctrl", "T"]] },
  { action: "Find in page", keys: [["Ctrl", "F"]] },
  { action: "Reader mode", detail: "Esc also leaves it", keys: [["Alt", "R"]] },
  { action: "Downloads", keys: [["Ctrl", "J"]] },
  { action: "Edit the address", keys: [["Ctrl", "L"]] },
  { action: "New window", keys: [["Ctrl", "N"]] },
  { action: "New private window", keys: [["Ctrl", "Shift", "N"]] },
  { action: "Cycle the sidebar", detail: "Full, rail, hidden", keys: [["Ctrl", "B"], ["Ctrl", "Shift", "S"]] },
  { action: "Settings", keys: [["Ctrl", ","]] },
  { action: "Split View", keys: [["Ctrl", "Shift", "\\"]] },
  { action: "Back", keys: [["Alt", "←"]] },
  { action: "Forward", keys: [["Alt", "→"]] },
  { action: "Reload", keys: [["Ctrl", "R"], ["F5"]] },
  { action: "Print", detail: "Or save as PDF", keys: [["Ctrl", "P"]] },
  { action: "Save page", keys: [["Ctrl", "S"]] },
  { action: "Developer tools", keys: [["F12"], ["Ctrl", "Shift", "I"]] },
  { action: "Close tab", detail: "Pinned tabs only unload", keys: [["Ctrl", "W"]] },
  { action: "Next tab", keys: [["Ctrl", "Tab"]] },
  { action: "Previous tab", keys: [["Ctrl", "Shift", "Tab"]] },
  { action: "Go to tab 1–8", keys: [["Ctrl", "1–8"]] },
  { action: "Go to the last tab", keys: [["Ctrl", "9"]] },
  { action: "Favorite the page", detail: "Press again to remove it", keys: [["Ctrl", "D"]] },
  { action: "Close a popover", keys: [["Esc"]] },
]

let state = null
let clearing = { phase: "idle" }
let importing = { phase: "idle" }
let confirmed = null
const typingTimers = new Map()

function byId(id) {
  return document.getElementById(id)
}

function post(message) {
  chrome.webview.postMessage(message)
}

function setSetting(key, value) {
  post({ type: "set_setting", key, value })
}

function onOff(checked) {
  return checked ? "On" : "Off"
}

function node(tag, { dataset = {}, ...properties } = {}, children = []) {
  const created = Object.assign(document.createElement(tag), properties)
  Object.assign(created.dataset, dataset)
  created.append(...children)
  return created
}

function translated(variant, fill = "") {
  return state?.text[variant]?.replace("{}", fill)
}

function localize(element) {
  const { text, placeholder, label, fill } = element.dataset
  const name = label ?? placeholder
  if (text) element.textContent = translated(text, fill) ?? element.textContent
  if (placeholder) element.placeholder = translated(placeholder) ?? element.placeholder
  if (name) element.ariaLabel = translated(name) ?? element.ariaLabel
}

function say(element, variant, fill = "") {
  element.dataset.text = variant
  element.dataset.fill = fill
  localize(element)
}

function write(element, ...content) {
  delete element.dataset.text
  element.replaceChildren(...content)
}

function plain(text, className) {
  return node("div", { className, textContent: text })
}

function phrase(variant, className) {
  const element = node("div", { className })
  say(element, variant)
  return element
}

function button({ variant, label, onclick }) {
  const created = node("button", { type: "button", className: "button", onclick })
  if (variant) say(created, variant)
  else created.textContent = label
  return created
}

function row({ title, description, controls = [], className = "row" }) {
  const label = node("div", { className: "label" }, [title, description].filter(Boolean))
  return node("div", { className }, [label, ...controls])
}

function renderList(container, items, build) {
  const signature = JSON.stringify(items)
  if (container.dataset.signature === signature) return
  container.dataset.signature = signature
  container.replaceChildren(...build(items))
}

function requirementMet(requires) {
  if (!requires) return true
  const [key, value] = requires.split("=")
  return state.settings[key] === value
}

function segmented(key, options) {
  const segments = options.map(({ value, name }) =>
    node("label", {}, [node("input", { type: "radio", name: key, value }), name]),
  )
  return node("div", { className: "segmented", role: "radiogroup" }, segments)
}

function dropdown(options) {
  const items = options.map(({ value, name }) => node("option", { value, textContent: name }))
  return node("select", { className: "select" }, items)
}

function buildChoice(container, options) {
  const key = container.dataset.setting
  const control = options.length <= SEGMENTED_LIMIT ? segmented(key, options) : dropdown(options)
  control.ariaLabel = container.closest(".row").querySelector(".title").textContent
  container.replaceChildren(control)
}

function renderChoice(container, options, value) {
  const signature = JSON.stringify(options)
  if (container.dataset.signature !== signature) {
    container.dataset.signature = signature
    buildChoice(container, options)
  }
  const select = container.querySelector("select")
  if (select) select.value = value
  for (const radio of container.querySelectorAll("input")) radio.checked = radio.value === value
}

function renderControl(control) {
  const key = control.dataset.setting
  const row = control.closest(".row")
  row.hidden = !(key in state.settings) || !requirementMet(row.dataset.requires)
  if (row.hidden) return
  if (control.classList.contains("choice")) return renderChoice(control, state.choices[key], state.settings[key])
  if (control.type === "checkbox") {
    control.checked = state.settings[key] === "On"
    return
  }
  if (control !== document.activeElement) control.value = state.settings[key]
}

function renderDescriptions() {
  say(byId("tracking-description"), TRACKING_DESCRIPTIONS[state.settings.tracking_prevention])
  const hint = byId("custom-search-hint")
  say(hint, state.custom_search_valid ? "SearchAddressHint" : "SearchAddressInvalid")
  hint.classList.toggle("error", !state.custom_search_valid)
}

function renderDownloads() {
  byId("downloads-folder").textContent = state.downloads_folder
  byId("download-reset").hidden = !state.settings.download_folder
}

function emptyRow(title, description) {
  return row({ title, description, className: "row empty" })
}

function zoomRows(zooms) {
  if (zooms.length === 0) {
    return [emptyRow(plain("No zoomed sites", "title"), plain("Sites you zoom with Ctrl + or Ctrl − keep their size here", "description"))]
  }
  return zooms.map(({ site, zoom }) => {
    const reset = button({ label: "Reset", onclick: () => post({ type: "clear_zoom", site }) })
    reset.ariaLabel = `Reset zoom for ${site}`
    const controls = node("div", { className: "actions" }, [node("span", { className: "value", textContent: zoom }), reset])
    return row({ title: plain(site, "title"), controls: [controls] })
  })
}

function pausedSiteRows(sites) {
  if (sites.length === 0) return [emptyRow(phrase("NoPausedSites", "title"), phrase("NoPausedSitesHint", "description"))]
  return sites.map(site => {
    const resume = button({ variant: "ResumeBlocking", onclick: () => post({ type: "unpause_site", site }) })
    return row({ title: plain(site, "title"), description: phrase("BlockingPaused", "description"), controls: [resume] })
  })
}

function darkSiteRows(sites) {
  if (sites.length === 0) return [emptyRow(phrase("NoDarkSites", "title"), phrase("NoDarkSitesHint", "description"))]
  return sites.map(({ site, mode }) => {
    const remove = button({ variant: "Remove", onclick: () => post({ type: "reset_dark_site", site }) })
    return row({ title: plain(site, "title"), description: phrase(DARK_SITE_MODES[mode], "description"), controls: [remove] })
  })
}

function setPermission(origin, kind, state) {
  post({ type: "set_permission", origin, kind, state })
}

function permissionChoice({ origin, kind, label, state: current }) {
  const options = state.choices.site_permission.map(({ value, name }) =>
    node("option", { value, textContent: name, selected: value === current }),
  )
  const select = node("select", { className: "select", onchange: ({ target }) => setPermission(origin, kind, target.value) }, options)
  select.ariaLabel = `${label} · ${origin}`
  return select
}

function sitePermissionRows(permissions) {
  if (permissions.length === 0) {
    return [emptyRow(phrase("NoSitePermissions", "title"), phrase("NoSitePermissionsHint", "description"))]
  }
  return permissions.map(permission => {
    const remove = button({ variant: "Remove", onclick: () => setPermission(permission.origin, permission.kind, "ask") })
    const controls = node("div", { className: "actions" }, [permissionChoice(permission), remove])
    return row({ title: plain(permission.origin, "title"), description: plain(permission.label, "description"), controls: [controls] })
  })
}

function categoryRows(categories) {
  return categories.map(({ key, label, description }) => {
    const text = node("span", { className: "label" }, [
      node("span", { className: "title", textContent: label }),
      node("span", { className: "description", textContent: description }),
    ])
    const toggle = node("input", { type: "checkbox", className: "switch", role: "switch", dataset: { sponsorblock: key } })
    return node("label", { className: "row" }, [text, toggle])
  })
}

function channelRows(channels) {
  return channels.map(channel => {
    const remove = button({ variant: "Remove", onclick: () => post({ type: "unpause_channel", channel }) })
    return row({ title: plain(channel, "title"), description: phrase("NothingSkipped", "description"), controls: [remove] })
  })
}

function renderSponsorBlock(sponsorblock) {
  const categories = sponsorblock.categories.map(({ key, label, description }) => ({ key, label, description }))
  renderList(byId("categories"), categories, categoryRows)
  renderList(byId("channels"), sponsorblock.channels, channelRows)
  byId("channels-group").hidden = sponsorblock.channels.length === 0
  const switches = new Map([
    ["enabled", sponsorblock.enabled],
    ["show_skip_notice", sponsorblock.show_skip_notice],
    ...sponsorblock.categories.map(({ key, on }) => [key, on]),
  ])
  for (const toggle of document.querySelectorAll("[data-sponsorblock]")) {
    toggle.checked = switches.get(toggle.dataset.sponsorblock)
  }
}

function renderClearing() {
  const view = CLEARING_VIEWS[clearing.phase]
  const description = byId("clear-description")
  say(byId("clear-title"), view.title)
  say(description, view.description, clearing.error)
  description.classList.toggle("success", view.tone === "success")
  description.classList.toggle("error", view.tone === "error")
  byId("clear-start").hidden = clearing.phase === "running"
}

function setClearing(next) {
  clearing = next
  renderClearing()
}

function chosenProfile() {
  return document.querySelector('input[name="import-profile"]:checked')?.value
}

function scopeChosen() {
  return byId("import-bookmarks").checked || byId("import-history").checked
}

function importReady() {
  return Boolean(chosenProfile()) && scopeChosen() && importing.phase !== "running"
}

function renderImport() {
  const summary = byId("import-summary")
  const status = byId("import-status")
  byId("import-run").disabled = !importReady()
  summary.hidden = importing.phase !== "done"
  switch (importing.phase) {
    case "running":
      return say(status, "Importing")
    case "done":
      summary.textContent = importing.summary
      return write(status, ...importing.warnings.map(warning => plain(warning)))
    default:
      return say(status, scopeChosen() ? "ImportAdds" : "ImportChoose")
  }
}

function importProfileRows(profiles) {
  if (profiles.length === 0) return [emptyRow(phrase("NoOtherBrowsers", "title"), phrase("NoOtherBrowsersHint", "description"))]
  return profiles.map(({ id, label }) =>
    node("label", { className: "row option" }, [
      node("input", { type: "radio", name: "import-profile", value: id, className: "check" }),
      node("span", { className: "label" }, [node("span", { className: "title", textContent: label })]),
    ]),
  )
}

function showImportProfiles(profiles) {
  const chosen = chosenProfile()
  renderList(byId("import-profiles"), profiles, importProfileRows)
  const radios = [...document.querySelectorAll('input[name="import-profile"]')]
  const radio = radios.find(({ value }) => value === chosen) ?? radios[0]
  if (radio) radio.checked = true
  byId("import-scope").hidden = profiles.length === 0
  renderImport()
}

function runImport() {
  post({
    type: "run_import",
    profile: chosenProfile(),
    bookmarks: byId("import-bookmarks").checked,
    history: byId("import-history").checked,
  })
  importing = { phase: "running" }
  renderImport()
}

function finishImport({ summary, warnings }) {
  importing = { phase: "done", summary, warnings }
  renderImport()
}

function showBackup({ status, message }) {
  const description = byId("backup-status")
  byId("backup-row").hidden = false
  description.textContent = message
  description.classList.toggle("success", status === "done")
  description.classList.toggle("error", status === "failed")
}

function changeImport() {
  if (importing.phase === "done") importing = { phase: "idle" }
  renderImport()
}

function profileNameField({ id, name }) {
  const rename = ({ target }) => {
    const next = target.value.trim()
    if (next && next !== name) post({ type: "rename_profile", profile: id, name: next })
    else target.value = name
  }
  const field = node("input", { className: "field", type: "text", spellcheck: false, maxLength: 40, value: name, onchange: rename })
  field.ariaLabel = state.text.Name
  return field
}

function ask({ question, fill = "", detail, action, run }) {
  say(byId("confirm-title"), question, fill)
  say(byId("confirm-description"), detail)
  say(byId("confirm-run"), action)
  confirmed = run
  byId("confirm").showModal()
}

function confirmProfileDeletion({ id, name }) {
  ask({
    question: "DeleteProfileConfirm",
    fill: name,
    detail: "DeleteProfileWarning",
    action: "DeleteProfileNow",
    run: () => post({ type: "delete_profile", profile: id }),
  })
}

function deleteProfileButton(profile) {
  const remove = button({ variant: "DeleteProfile", onclick: () => confirmProfileDeletion(profile) })
  remove.classList.add("danger")
  return remove
}

function profileRows(profiles) {
  return profiles.map(profile => {
    if (profile.built_in) {
      return row({ title: plain(profile.name, "title"), description: phrase("DefaultProfileHint", "description") })
    }
    return row({ title: profileNameField(profile), controls: [deleteProfileButton(profile)] })
  })
}

function renderProfiles() {
  renderList(byId("browsing-profiles"), state.profiles, profileRows)
}

function profileChoice({ id, name, profile }) {
  const options = state.profiles.map(choice =>
    node("option", { value: choice.id, textContent: choice.name, selected: choice.id === profile }),
  )
  const assign = ({ target }) => post({ type: "assign_profile", space: id, profile: target.value })
  const select = node("select", { className: "select", onchange: assign }, options)
  select.ariaLabel = `${state.text.SpaceProfiles} · ${name}`
  return select
}

function spaceProfileRows({ spaces }) {
  return spaces.map(space => row({ title: plain(space.name, "title"), controls: [profileChoice(space)] }))
}

function renderSpaceProfiles() {
  renderList(byId("space-profiles"), { spaces: state.spaces, profiles: state.profiles }, spaceProfileRows)
  byId("space-profiles-group").hidden = state.spaces.length === 0
}

function addProfile(event) {
  event.preventDefault()
  const field = byId("profile-name")
  const name = field.value.trim()
  if (!name) return
  post({ type: "create_profile", name })
  field.value = ""
}

function updateMessage(update) {
  switch (update.state) {
    case "idle":
      return "Magpie looks for new releases on GitHub"
    case "checking":
      return "Checking for updates…"
    case "up_to_date":
      return "Magpie is up to date"
    case "failed":
      return `Could not check for updates. ${update.message}`
    case "available":
      return `Update available: v${update.offer.version}`
    case "downloading":
      return `Downloading v${update.offer.version}… ${update.percent}%`
    case "download_failed":
      return `Could not install v${update.offer.version}. ${update.message}`
    case "installing":
      return `Installing v${update.offer.version}…`
    case "install_failed":
      return `The last update didn't install. ${update.message}`
  }
}

function renderUpdates() {
  const update = state.update
  const offer = update.offer
  const description = byId("update-status")
  const link = byId("update-link")
  const notes = byId("update-notes")
  const installable = INSTALLABLE_UPDATES.includes(update.state) && offer.download !== null
  description.textContent = updateMessage(update)
  description.classList.toggle("success", UPDATE_TONES[update.state] === "success")
  description.classList.toggle("error", UPDATE_TONES[update.state] === "error")
  byId("update-check").disabled = BUSY_UPDATES.includes(update.state)
  byId("update-install").hidden = !installable
  link.hidden = !offer
  link.href = offer?.page ?? ""
  notes.textContent = update.state === "available" ? offer.notes : ""
  notes.hidden = !notes.textContent
}

function versionLabel({ version, commit }) {
  const nightly = NIGHTLY_VERSION.exec(version)
  if (!nightly) return `Version ${version}`
  const [, base, year, month, day, hour, minute] = nightly
  const build = [`${year}-${month}-${day} ${hour}:${minute} UTC`, commit].filter(Boolean).join(" · ")
  return `Version ${base} (Nightly ${build})`
}

function renderAbout() {
  byId("version").textContent = versionLabel(state)
  byId("data-folder").textContent = state.data_folder
  renderUpdates()
}

function vault(action) {
  post({ type: "passwords", action })
}

function vaultDetail(passwords) {
  if (passwords.status === "signed_out") return state.text.BitwardenHint
  return passwords.email ?? passwords.reason ?? ""
}

function chosenMethod() {
  return byId("vault-methods").querySelector("input:checked")?.value
}

function renderMethods({ methods, email_sent }) {
  const methodsElement = byId("vault-methods")
  const signature = JSON.stringify(methods)
  if (methodsElement.dataset.signature !== signature) {
    methodsElement.dataset.signature = signature
    methodsElement.replaceChildren(...segmented("vault-method", methods).children)
    methodsElement.querySelector("input").checked = true
  }
  methodsElement.hidden = methods.length < 2
  const emailCode = byId("vault-email-code")
  emailCode.hidden = !EMAILED_METHODS.includes(chosenMethod())
  say(emailCode, email_sent ? "EmailCodeSent" : "SendEmailCode")
}

function renderPasswords(passwords) {
  const { status } = passwords
  say(byId("vault-status"), VAULT_TITLES[status])
  const detail = byId("vault-detail")
  detail.textContent = vaultDetail(passwords)
  detail.hidden = !detail.textContent
  detail.classList.toggle("error", status === "unavailable")
  const problem = byId("vault-problem")
  problem.textContent = passwords.problem ?? ""
  problem.hidden = !problem.textContent
  byId("vault-sign-in").hidden = status !== "signed_out"
  byId("vault-challenge").hidden = status !== "challenged"
  byId("vault-unlock").hidden = status !== "locked"
  byId("vault-browse").hidden = status !== "unlocked"
  byId("vault-sync").hidden = status !== "unlocked"
  byId("vault-lock").hidden = status !== "unlocked"
  byId("vault-sign-out").hidden = !SIGNED_IN.includes(status)
  if (status === "challenged") renderMethods(passwords.challenge)
  if (status !== "unlocked") return
  say(byId("vault-count"), "SavedLogins", String(passwords.logins))
  searchVault()
}

function searchVault() {
  vault({ kind: "search", query: byId("vault-search").value })
}

function copyButton(login, part, variant) {
  return button({ variant, onclick: () => vault({ kind: "copy", id: login.id, part }) })
}

function vaultRows(logins) {
  if (logins.length === 0) return [row({ title: phrase("NoMatchingLogins", "title"), className: "row empty" })]
  return logins.map((login) => {
    const copies = [
      login.username && copyButton(login, "username", "Username"),
      login.password && copyButton(login, "password", "Password"),
      login.code && copyButton(login, "code", "Code"),
    ].filter(Boolean)
    const detail = [login.username, login.site].filter(Boolean).join(" · ")
    return row({
      title: plain(login.name, "title"),
      description: plain(detail, "description"),
      controls: [node("div", { className: "actions" }, copies)],
    })
  })
}

function confirmSignOut() {
  ask({ question: "SignOutConfirm", detail: "SignOutWarning", action: "SignOut", run: () => vault({ kind: "sign_out" }) })
}

function signIn(event) {
  event.preventDefault()
  const host = document.querySelector("input[name=vault-host]:checked").value
  const password = byId("vault-password")
  vault({ kind: "sign_in", host, address: byId("vault-address").value, email: byId("vault-email").value, password: password.value })
  password.value = ""
}

function answerChallenge(event) {
  event.preventDefault()
  const code = byId("vault-code")
  vault({ kind: "answer_challenge", method: chosenMethod(), code: code.value })
  code.value = ""
}

function unlockVault(event) {
  event.preventDefault()
  const password = byId("vault-unlock-password")
  vault({ kind: "unlock", password: password.value })
  password.value = ""
}

function chooseHost() {
  const host = document.querySelector("input[name=vault-host]:checked").value
  byId("vault-address").hidden = host !== "custom"
}

function keyCombo(keys) {
  return keys.map(key => node("kbd", { textContent: key }))
}

function shortcutRows() {
  return SHORTCUTS.map(({ action, detail, keys }) => {
    const combos = keys.flatMap((combo, index) => [
      ...(index > 0 ? [node("span", { className: "or", textContent: "or" })] : []),
      ...keyCombo(combo),
    ])
    const description = detail && plain(detail, "description")
    const keysElement = node("div", { className: "keys" }, combos)
    return row({ title: plain(action, "title"), description, controls: [keysElement], className: "row shortcut" })
  })
}

function applyState(next) {
  state = next
  document.documentElement.dataset.scheme = state.scheme
  for (const element of document.querySelectorAll(LOCALIZED)) localize(element)
  for (const control of document.querySelectorAll("[data-setting]")) renderControl(control)
  renderDescriptions()
  renderDownloads()
  renderList(byId("zooms"), state.zooms, zoomRows)
  renderList(byId("paused-sites"), state.paused_sites, pausedSiteRows)
  renderList(byId("dark-sites"), state.dark_sites, darkSiteRows)
  renderList(byId("site-permissions"), state.permissions, sitePermissionRows)
  renderSponsorBlock(state.sponsorblock)
  renderClearing()
  renderImport()
  renderAbout()
  renderPasswords(state.passwords)
  renderProfiles()
  renderSpaceProfiles()
  refreshMatches()
}

function received({ data }) {
  switch (data.type) {
    case "state":
      return applyState(data)
    case "import_profiles":
      return showImportProfiles(data.profiles)
    case "cleared":
      return setClearing(data.error === null ? { phase: "cleared" } : { phase: "failed", error: data.error })
    case "imported":
      return finishImport(data)
    case "backup":
      return showBackup(data)
    case "logins":
      return renderList(byId("vault-logins"), data.logins, vaultRows)
  }
}

function commit(field) {
  clearTimeout(typingTimers.get(field))
  const key = field.dataset.setting
  const value = field.value.trim()
  if (!field.checkValidity() || value === state.settings[key]) return
  setSetting(key, value)
}

function typed({ target }) {
  if (!target.dataset.setting || target.type === "checkbox") return
  clearTimeout(typingTimers.get(target))
  typingTimers.set(target, setTimeout(() => commit(target), TYPING_PAUSE_MS))
}

function changed({ target }) {
  const sponsorblockKey = target.dataset.sponsorblock
  if (sponsorblockKey) return post({ type: "set_sponsorblock", key: sponsorblockKey, value: onOff(target.checked) })
  const control = target.closest("[data-setting]")
  if (!control) return
  const key = control.dataset.setting
  if (control !== target) return setSetting(key, target.value)
  if (target.type === "checkbox") return setSetting(key, onOff(target.checked))
  commit(target)
}

function sections() {
  return [...document.querySelectorAll(".section")]
}

function currentSection() {
  const id = location.hash.slice(1)
  return sections().some(section => section.dataset.section === id) ? id : DEFAULT_SECTION
}

function queryWords() {
  return byId("find").value.toLowerCase().split(/\s+/).filter(Boolean)
}

function matchesAll(text, words) {
  const haystack = text.toLowerCase()
  return words.every(word => haystack.includes(word))
}

function filterGroup(group, context, words) {
  const heading = group.querySelector("h2")?.textContent ?? ""
  const hits = [...group.querySelectorAll(".row")].map(row => {
    const hit = !row.hidden && matchesAll(`${context} ${heading} ${row.textContent}`, words)
    row.classList.toggle("unmatched", !hit)
    return hit
  })
  const shown = !group.hidden && hits.some(Boolean)
  group.classList.toggle("unmatched", !shown)
  return shown
}

function filterSection(section, words) {
  const title = section.querySelector("h1").textContent
  const shown = [...section.querySelectorAll(".group")].map(group => filterGroup(group, title, words)).some(Boolean)
  section.hidden = !shown
  return shown
}

function showMatches(words) {
  byId("main").classList.add("searching")
  const found = sections().map(section => filterSection(section, words)).some(Boolean)
  byId("no-results-query").textContent = byId("find").value.trim()
  byId("no-results").hidden = found
}

function showSection(id) {
  byId("main").classList.remove("searching")
  for (const unmatched of document.querySelectorAll(".unmatched")) unmatched.classList.remove("unmatched")
  for (const section of sections()) section.hidden = section.dataset.section !== id
  for (const link of document.querySelectorAll(".rail a")) link.ariaCurrent = link.dataset.section === id ? "page" : null
  byId("no-results").hidden = true
}

function showView() {
  const words = queryWords()
  if (words.length > 0) showMatches(words)
  else showSection(currentSection())
  window.scrollTo(0, 0)
}

function refreshMatches() {
  const words = queryWords()
  if (words.length > 0) showMatches(words)
}

function navigate(event) {
  const link = event.target.closest("a[data-section]")
  if (!link) return
  byId("find").value = ""
  if (link.hash === location.hash) showView()
}

function clearFindOnEscape(event) {
  if (event.key !== "Escape") return
  event.target.value = ""
  showView()
}

function clearBrowsingData() {
  post({ type: "clear_browsing_data" })
  setClearing({ phase: "running" })
}

function confirmClearing() {
  ask({ question: "ClearBrowsingDataConfirm", detail: "ClearBrowsingDataWarning", action: "ClearNow", run: clearBrowsingData })
}

function confirmBackupImport() {
  ask({
    question: "ImportBackupConfirm",
    detail: "ImportBackupWarning",
    action: "Continue",
    run: () => post({ type: "import_backup" }),
  })
}

function listen() {
  document.addEventListener("change", changed)
  document.addEventListener("input", typed)
  window.addEventListener("hashchange", showView)
  byId("rail").addEventListener("click", navigate)
  byId("find").addEventListener("input", showView)
  byId("find").addEventListener("keydown", clearFindOnEscape)
  byId("import").addEventListener("change", changeImport)
  byId("import-run").addEventListener("click", runImport)
  byId("backup-export").addEventListener("click", () => post({ type: "export_backup" }))
  byId("backup-import").addEventListener("click", confirmBackupImport)
  byId("profile-add").addEventListener("submit", addProfile)
  byId("clear-start").addEventListener("click", confirmClearing)
  byId("confirm-run").addEventListener("click", () => confirmed())
  byId("download-change").addEventListener("click", () => post({ type: "choose_download_folder" }))
  byId("download-reset").addEventListener("click", () => setSetting("download_folder", ""))
  byId("open-data-folder").addEventListener("click", () => post({ type: "open_data_folder" }))
  byId("update-check").addEventListener("click", () => post({ type: "check_for_updates" }))
  byId("update-install").addEventListener("click", () => post({ type: "install_update" }))
  byId("vault-sign-in").addEventListener("submit", signIn)
  byId("vault-sign-in").addEventListener("change", chooseHost)
  byId("vault-challenge").addEventListener("submit", answerChallenge)
  byId("vault-methods").addEventListener("change", () => renderMethods(state.passwords.challenge))
  byId("vault-email-code").addEventListener("click", () => vault({ kind: "send_email_code" }))
  byId("vault-cancel").addEventListener("click", () => vault({ kind: "cancel_sign_in" }))
  byId("vault-unlock").addEventListener("submit", unlockVault)
  byId("vault-sync").addEventListener("click", () => vault({ kind: "sync" }))
  byId("vault-lock").addEventListener("click", () => vault({ kind: "lock" }))
  byId("vault-sign-out").addEventListener("click", confirmSignOut)
  byId("vault-search").addEventListener("input", searchVault)
  chrome.webview.addEventListener("message", received)
}

function start() {
  byId("shortcuts").replaceChildren(...shortcutRows())
  listen()
  showView()
  post({ type: "ready" })
}

start()
