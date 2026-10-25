// Интерфейс общается только с Rust-слоем Tauri через invoke;
// HTTP-запросы к backend выполняют команды на стороне Rust.
const { invoke } = window.__TAURI__.core;

const KIND_LABELS = {
  socket: "Розетка",
  thermometer: "Термометр",
  light: "Светильник",
};

const STATUS_LABELS = {
  on: "Включено",
  off: "Выключено",
  offline: "Не в сети",
};

const ERROR_TITLES = {
  unavailable: "Backend недоступен",
  not_found: "Не найдено",
  validation: "Ошибка в данных",
  conflict: "Конфликт",
  server: "Ошибка backend",
  config: "Ошибка конфигурации",
  unexpected: "Непредвиденная ошибка",
};

const state = {
  rooms: [],
  roomId: null,
  deviceId: null,
};

// Номера последних запросов по панелям: ответ на устаревший запрос
// (пользователь уже выбрал другую комнату) просто отбрасывается.
const requests = { rooms: 0, room: 0, devices: 0, device: 0, report: 0 };

const $ = (selector) => document.querySelector(selector);

/** Создает элемент; текст задается только через textContent. */
function el(tag, props = {}, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(props)) {
    if (key === "class") node.className = value;
    else if (key === "text") node.textContent = value;
    else if (key.startsWith("on")) node.addEventListener(key.slice(2), value);
    else node.setAttribute(key, value);
  }
  for (const child of children.flat()) {
    if (child == null) continue;
    node.append(child instanceof Node ? child : document.createTextNode(String(child)));
  }
  return node;
}

async function call(command, args) {
  try {
    return await invoke(command, args);
  } catch (error) {
    if (error && typeof error === "object" && "message" in error) throw error;
    throw { kind: "unexpected", message: String(error) };
  }
}

function errorTitle(error) {
  return ERROR_TITLES[error.kind] ?? ERROR_TITLES.unexpected;
}

function loadingView(text) {
  return el("div", { class: "loading" }, el("span", { class: "spinner" }), text);
}

function errorView(error, retry) {
  return el(
    "div",
    { class: "error-box", role: "alert" },
    el("div", {}, el("strong", { text: errorTitle(error) }), error.message),
    retry && el("button", { type: "button", class: "danger", onclick: retry, text: "Повторить" }),
  );
}

function statusBadge(status) {
  return el("span", { class: `badge ${status}`, text: STATUS_LABELS[status] ?? status });
}

function showToast(error) {
  $("#toast-text").textContent = `${errorTitle(error)}: ${error.message}`;
  $("#toast").hidden = false;
}

function hideToast() {
  $("#toast").hidden = true;
}

/** Блокирует кнопку на время операции и показывает ход выполнения. */
async function withBusy(button, busyText, action) {
  const original = button.textContent;
  button.disabled = true;
  button.textContent = busyText;
  try {
    return await action();
  } finally {
    button.disabled = false;
    button.textContent = original;
  }
}

/** Удаление требует второго нажатия в течение трех секунд. */
function confirmButton(text, onConfirm) {
  let timer = null;
  const button = el("button", { type: "button", class: "danger", text });
  button.addEventListener("click", async () => {
    if (timer === null) {
      button.textContent = "Точно удалить?";
      timer = setTimeout(() => {
        timer = null;
        button.textContent = text;
      }, 3000);
      return;
    }
    clearTimeout(timer);
    timer = null;
    button.textContent = text;
    await withBusy(button, "Удаление…", onConfirm);
  });
  return button;
}

function setFormError(form, error) {
  const box = form.querySelector(".form-error");
  if (error) {
    box.textContent = `${errorTitle(error)}: ${error.message}`;
    box.hidden = false;
  } else {
    box.hidden = true;
  }
}

// ---------- Комнаты ----------

async function loadRooms() {
  const id = ++requests.rooms;
  const list = $("#rooms-list");
  list.replaceChildren(loadingView("Загрузка комнат…"));
  try {
    const rooms = await call("list_rooms");
    if (id !== requests.rooms) return;
    state.rooms = rooms;
    renderRooms();
    if (state.roomId !== null && !rooms.some((room) => room.id === state.roomId)) {
      selectRoom(null);
    }
  } catch (error) {
    if (id !== requests.rooms) return;
    state.rooms = [];
    list.replaceChildren(errorView(error, loadRooms));
  }
}

function renderRooms() {
  const list = $("#rooms-list");
  if (state.rooms.length === 0) {
    list.replaceChildren(el("div", { class: "empty", text: "Комнат пока нет" }));
    return;
  }
  list.replaceChildren(
    ...state.rooms.map((room) =>
      el(
        "button",
        {
          type: "button",
          class: `list-item${room.id === state.roomId ? " selected" : ""}`,
          onclick: () => selectRoom(room.id),
        },
        el("span", { text: room.name }),
        el("span", { class: "meta", text: `${room.device_count} устр.` }),
      ),
    ),
  );
}

function selectRoom(roomId) {
  state.roomId = roomId;
  selectDevice(null);
  renderRooms();
  if (roomId === null) {
    requests.room++;
    requests.devices++;
    $("#room-panel").replaceChildren(
      el("div", { class: "placeholder", text: "Выберите комнату слева или добавьте новую" }),
    );
  } else {
    loadRoom();
  }
}

async function addRoom(event) {
  event.preventDefault();
  const form = event.currentTarget;
  const input = form.elements.name;
  setFormError(form, null);
  try {
    const room = await withBusy(form.querySelector("button"), "Добавление…", () =>
      call("create_room", { name: input.value }),
    );
    input.value = "";
    await loadRooms();
    selectRoom(room.id);
  } catch (error) {
    setFormError(form, error);
  }
}

// ---------- Выбранная комната ----------

async function loadRoom() {
  const id = ++requests.room;
  const roomId = state.roomId;
  const panel = $("#room-panel");
  panel.replaceChildren(loadingView("Загрузка комнаты…"));
  try {
    const room = await call("get_room", { roomId });
    if (id !== requests.room) return;
    renderRoom(room);
    loadDevices();
  } catch (error) {
    if (id !== requests.room) return;
    panel.replaceChildren(errorView(error, loadRoom));
  }
}

function renderRoom(room) {
  const form = $("#device-form-template").content.firstElementChild.cloneNode(true);
  form.addEventListener("submit", addDevice);

  $("#room-panel").replaceChildren(
    el(
      "div",
      { class: "title-row" },
      el(
        "div",
        {},
        el("h2", { text: room.name }),
        el("div", { class: "subtitle", text: `id ${room.id} · устройств: ${room.devices.length}` }),
      ),
      confirmButton("Удалить комнату", () => deleteRoom(room.id)),
    ),
    el("h3", { text: "Устройства" }),
    el("div", { id: "devices-list", class: "list", "aria-live": "polite" }),
    form,
  );
}

async function deleteRoom(roomId) {
  try {
    await call("delete_room", { roomId });
    selectRoom(null);
    await loadRooms();
  } catch (error) {
    showToast(error);
  }
}

// ---------- Устройства комнаты ----------

async function loadDevices() {
  const id = ++requests.devices;
  const roomId = state.roomId;
  const list = $("#devices-list");
  if (!list) return;
  list.replaceChildren(loadingView("Загрузка устройств…"));
  try {
    const devices = await call("list_devices", { roomId });
    if (id !== requests.devices) return;
    renderDevices(devices);
  } catch (error) {
    if (id !== requests.devices) return;
    list.replaceChildren(errorView(error, loadDevices));
  }
}

function renderDevices(devices) {
  const list = $("#devices-list");
  if (devices.length === 0) {
    list.replaceChildren(el("div", { class: "empty", text: "В комнате нет устройств" }));
    return;
  }
  list.replaceChildren(
    ...devices.map((device) =>
      el(
        "button",
        {
          type: "button",
          class: `list-item${device.id === state.deviceId ? " selected" : ""}`,
          "data-device-id": device.id,
          onclick: () => selectDevice(device.id),
        },
        el(
          "span",
          {},
          el("div", { text: device.name }),
          el("div", { class: "meta", text: KIND_LABELS[device.kind] ?? device.kind }),
        ),
        statusBadge(device.status),
      ),
    ),
  );
}

async function addDevice(event) {
  event.preventDefault();
  const form = event.currentTarget;
  const { name, kind, status } = form.elements;
  setFormError(form, null);
  try {
    const device = await withBusy(form.querySelector("button"), "Добавление…", () =>
      call("create_device", {
        roomId: state.roomId,
        name: name.value,
        kind: kind.value,
        status: status.value,
      }),
    );
    name.value = "";
    state.deviceId = device.id;
    await Promise.all([loadRoom(), loadRooms()]);
    selectDevice(device.id);
  } catch (error) {
    setFormError(form, error);
  }
}

// ---------- Устройство ----------

function selectDevice(deviceId) {
  state.deviceId = deviceId;
  for (const item of document.querySelectorAll("#devices-list .list-item")) {
    item.classList.toggle("selected", Number(item.dataset.deviceId) === deviceId);
  }
  if (deviceId === null) {
    requests.device++;
    $("#device-panel").replaceChildren(
      el("div", { class: "placeholder", text: "Выберите устройство, чтобы увидеть подробности" }),
    );
  } else {
    loadDevice();
  }
}

async function loadDevice() {
  const id = ++requests.device;
  const { roomId, deviceId } = state;
  const panel = $("#device-panel");
  panel.replaceChildren(loadingView("Загрузка устройства…"));
  try {
    const device = await call("get_device", { roomId, deviceId });
    if (id !== requests.device) return;
    renderDevice(device);
  } catch (error) {
    if (id !== requests.device) return;
    panel.replaceChildren(errorView(error, loadDevice));
  }
}

function renderDevice(device) {
  const room = state.rooms.find((r) => r.id === device.room_id);
  $("#device-panel").replaceChildren(
    el("h2", { text: device.name }),
    el(
      "dl",
      { class: "props" },
      el("dt", { text: "Идентификатор" }),
      el("dd", { text: device.id }),
      el("dt", { text: "Тип" }),
      el("dd", { text: KIND_LABELS[device.kind] ?? device.kind }),
      el("dt", { text: "Состояние" }),
      el("dd", {}, statusBadge(device.status)),
      el("dt", { text: "Комната" }),
      el("dd", { text: room ? room.name : `id ${device.room_id}` }),
    ),
    el("div", {}, confirmButton("Удалить устройство", () => deleteDevice(device))),
  );
}

async function deleteDevice(device) {
  try {
    await call("delete_device", { roomId: device.room_id, deviceId: device.id });
    selectDevice(null);
    await Promise.all([loadRoom(), loadRooms()]);
  } catch (error) {
    showToast(error);
  }
}

// ---------- Отчет ----------

async function openReport() {
  const dialog = $("#report-dialog");
  if (!dialog.open) dialog.showModal();
  const id = ++requests.report;
  const body = $("#report-body");
  body.replaceChildren(loadingView("Формирование отчета…"));
  try {
    const report = await call("house_report");
    if (id !== requests.report) return;
    renderReport(report);
  } catch (error) {
    if (id !== requests.report) return;
    body.replaceChildren(errorView(error, openReport));
  }
}

function countTable(title, counts, labels) {
  return el(
    "div",
    { class: "report-section" },
    el("h3", { text: title }),
    el(
      "table",
      {},
      el(
        "tbody",
        {},
        Object.entries(counts).map(([key, count]) =>
          el("tr", {}, el("td", { text: labels[key] ?? key }), el("td", { class: "num", text: count })),
        ),
      ),
    ),
  );
}

function renderReport(report) {
  const stat = (value, label) =>
    el("div", { class: "stat" }, el("div", { class: "value", text: value }), el("div", { class: "label", text: label }));

  const rooms =
    report.rooms.length === 0
      ? el("div", { class: "empty", text: "Комнат нет" })
      : el(
          "table",
          {},
          el("thead", {}, el("tr", {}, el("th", { text: "Комната" }), el("th", { class: "num", text: "Устройств" }))),
          el(
            "tbody",
            {},
            report.rooms.map((room) =>
              el("tr", {}, el("td", { text: room.name }), el("td", { class: "num", text: room.device_count })),
            ),
          ),
        );

  $("#report-body").replaceChildren(
    el("div", { class: "stats" }, stat(report.room_count, "комнат"), stat(report.device_count, "устройств")),
    countTable("Устройства по типам", report.devices_by_kind, KIND_LABELS),
    countTable("Устройства по состояниям", report.devices_by_status, STATUS_LABELS),
    el("div", { class: "report-section" }, el("h3", { text: "Комнаты" }), rooms),
  );
}

// ---------- Запуск ----------

async function refreshAll() {
  hideToast();
  await loadRooms();
  if (state.roomId !== null) {
    const deviceId = state.deviceId;
    await loadRoom();
    if (deviceId !== null) selectDevice(deviceId);
  }
}

async function init() {
  $("#room-form").addEventListener("submit", addRoom);
  $("#refresh").addEventListener("click", refreshAll);
  $("#open-report").addEventListener("click", openReport);
  $("#report-close").addEventListener("click", () => $("#report-dialog").close());
  $("#toast-close").addEventListener("click", hideToast);

  try {
    $("#backend-url").textContent = await call("backend_url");
  } catch (error) {
    showToast(error);
  }
  await loadRooms();
}

init();
