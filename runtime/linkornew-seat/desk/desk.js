var actions = { console: true, browser: true, "command-post": true };

function openAction(action) {
  var notice = document.getElementById("notice");
  if (!actions[action]) {
    notice.textContent = "refuse " + action;
    return;
  }
  notice.textContent = "start " + action;
  var seat = window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.linkornew;
  if (seat && typeof seat.postMessage === "function") {
    try {
      seat.postMessage(action);
    } catch (err) {
      notice.textContent = "handler fail " + action;
    }
  }
  var probe = new Image();
  probe.onerror = function () {
    if (notice.textContent === "start " + action) {
      notice.textContent = "launch-uri fail " + action;
    }
  };
  probe.src = "linkornew://launch/" + action;
}

Array.prototype.forEach.call(document.querySelectorAll("button[data-action]"), function (button) {
  button.addEventListener("click", function () {
    openAction(button.getAttribute("data-action"));
  });
});

document.addEventListener("keydown", function (event) {
  if (event.key !== "Enter" && event.key !== " ") {
    return;
  }
  var el = document.activeElement;
  if (!el || !el.getAttribute) {
    return;
  }
  var action = el.getAttribute("data-action");
  if (!action) {
    return;
  }
  event.preventDefault();
  openAction(action);
});
