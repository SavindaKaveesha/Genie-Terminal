const { invoke } = window.__TAURI__.core;

const body = document.getElementsByTagName("body")[0];
const commandInputEl = document.getElementById("command-user-input");
const commandsDropDown = document.getElementById("commandsDropDown");
const commandOutputRoot = document.getElementById("commandOutputRoot");
const terminal = document.querySelector(".terminal");
const promptDisplay = document.getElementById("prompt-display");

let cwd = "C:\\";
const rootCwd = cwd;

async function getSuggestions() {
  await invoke("get_suggestions", { name: commandInputEl.value })
    .then((commands) => {
      commandsDropDown.innerHTML = "";

      for (let key in commands) {
        let spanElement = document.createElement("span");
        spanElement.id = key;
        spanElement.innerHTML = "<p>" + key + "</p><span>" + commands[key] + "</span>";
        spanElement.addEventListener("click", () => {
          commandInputEl.value = key;
          commandsDropDown.style.display = "none";
        });

        commandsDropDown.appendChild(spanElement);
      }

      commandsDropDown.style.display = "grid";
      let firstChild = commandsDropDown.firstElementChild;
      
      if(firstChild !== null) {
        firstChild.classList.add("dropdown-detect");
      }

    });
}

window.addEventListener("DOMContentLoaded", () => {

  //initial prompt display value
  promptDisplay.innerHTML = cwd + ">";

  commandInputEl.addEventListener("keydown", (e) => {  
    let spanEl = document.getElementsByClassName("dropdown-detect")[0];

    if (spanEl && spanEl.hasChildNodes() && e.key === "Tab"){
      e.preventDefault();
      let pEl = spanEl.firstElementChild;
      commandInputEl.value = pEl.innerHTML;
      commandsDropDown.style.display = "none";
    } else if (spanEl && spanEl.nextSibling !== null && e.key === "ArrowDown") {
      spanEl.classList.remove("dropdown-detect");
      spanEl.nextSibling.classList.add("dropdown-detect");
    } else if (spanEl && spanEl.previousSibling !== null && e.key === "ArrowUp") {
      spanEl.classList.remove("dropdown-detect");
      spanEl.previousSibling.classList.add("dropdown-detect");
    }

    commandsDropDown.style.marginInlineStart = 10 * commandInputEl.selectionStart + "px";
    commandsDropDown.style.marginInlineEnd = -10 * commandInputEl.selectionStart + "px";
  });

  commandInputEl.addEventListener("keyup", function (event) {

    if(event.key === "ArrowDown" || event.key === "ArrowUp" || event.key === "Tab") {
      return;
    } 

    if (event.key === "Enter") {
      const command = commandInputEl.value.trim();
      commandInputEl.value = "";    

      invoke("print_cmd_output", { name: command, cwd: cwd }).then(function (result) {
        const output_header = document.createElement("div");
        output_header.classList.add("command-output-header");
        
        output_header.innerText = cwd+"> "+command;
        commandOutputRoot.appendChild(output_header);

        const output = document.createElement("div");
        output.classList.add("command-output");
        output.innerText = result.output;
        commandOutputRoot.appendChild(output);

        if(cwd.endsWith("\\") && cwd !== rootCwd) {
          cwd = cwd.substring(0, cwd.length - 1);
        }

        if (command === "cd .." && cwd !== rootCwd) {     
          cwd = cwd.substring(0, cwd.lastIndexOf('\\')); 
        } else if(command.substring(0, 3) === "cd " && !result.output.includes("The system cannot find the path specified.")) {
          cwd = cwd + "\\" + command.substring(3);
        }

        //setting the prompt display
        promptDisplay.innerHTML = cwd + ">";

      });
      commandsDropDown.style.display = "none";
      
    } else if (event.target.value == "") {
      commandsDropDown.style.display = "none";
    } else {
      getSuggestions();
    }
  });

  commandOutputRoot.addEventListener('DOMNodeInserted', function ( event ) {
    body.scrollIntoView({ behavior: "smooth", block: "end", inline: "nearest" });
  }, false );

  body.addEventListener("click", () => commandInputEl.focus());

});
