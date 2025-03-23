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

// Create a MutationObserver to observe changes in the commandOutputRoot
const observer = new MutationObserver((mutationsList) => {
  for (const mutation of mutationsList) {
    if (mutation.type === 'childList') {
      body.scrollIntoView({ behavior: "smooth", block: "end", inline: "nearest" });
    }
  }
});

// Configure the observer to watch for child nodes being added
observer.observe(commandOutputRoot, { childList: true, subtree: true });


  body.addEventListener("click", () => commandInputEl);

});



const openChatbotBtn = document.getElementById("openChatbotBtn");
const closeChatbotBtn = document.getElementById("closeChatbotBtn");
const chatbotContainer = document.getElementById("chatbotContainer");
const chatbotMessages = document.getElementById("chatbotMessages");
const chatbotInput = document.getElementById("chatbotInput");
const sendChatbotMessage = document.getElementById("sendChatbotMessage");

// Open and close chatbot
openChatbotBtn.addEventListener("click", () => {
  chatbotContainer.style.right = "0";
  openChatbotBtn.style.display = "none";
});
closeChatbotBtn.addEventListener("click", () => {
  chatbotContainer.style.right = "-300px";
  openChatbotBtn.style.display = "block";
});

// Send a message to the chatbot API
sendChatbotMessage.addEventListener("click", async () => {
  const userMessage = chatbotInput.value.trim();
  if (!userMessage) return;

  // Display user message
  appendMessage(userMessage, "user");

  chatbotInput.value = "";

  // Call chatbot API
  const botResponse = await invokeChatbotAPI(userMessage);

  // Format the bot's response before appending it
  const formattedResponse = formatBotResponse(botResponse);

  // Append the formatted bot message
  appendMessage(formattedResponse, "bot");
});

// Append messages to the chatbot
function appendMessage(message, sender) {
  const messageElement = document.createElement("p");
  messageElement.classList.add(sender);
  messageElement.innerHTML = message; // Use innerHTML to render HTML tags like <b> and <br>
  chatbotMessages.appendChild(messageElement);
  chatbotMessages.scrollTop = chatbotMessages.scrollHeight; // Auto-scroll
}

// Call the chatbot API
async function invokeChatbotAPI(message) {
  try {
    const response = await fetch("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=AIzaSyC-0W1tfqreK-I30O5DJMb_DoduKbAFeis", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        contents: [{
          parts: [{
            text: message
          }]
        }]
      }),
    });

    if (!response.ok) {
      throw new Error("API response not OK");
    }

    const data = await response.json();

    if (data && Array.isArray(data.candidates) && data.candidates.length > 0) {
      const firstCandidate = data.candidates[0];
      // Accessing the 'text' property from the response
      if (firstCandidate.content && firstCandidate.content.parts && firstCandidate.content.parts[0].text) {
        return firstCandidate.content.parts[0].text;  // Return the 'text' from the first part
      } else {
        console.error("API response does not contain 'text' in parts", data);
        return "Sorry, the response format is not as expected.";
      }
    } else {
      console.error("Unexpected API response:", data);
      return "Sorry, something went wrong with the response.";
    }
  } catch (error) {
    console.error("Chatbot API error:", error);
    return "Sorry, something went wrong.";
  }
}

// Format the bot response to apply bold, code blocks, and line breaks
function formatBotResponse(response) {
  let responseArray = response.split("`"); // Split by backticks for code parts
  let newResponse = "";

  // Loop through the split response and apply code block formatting
  for (let i = 0; i < responseArray.length; i++) {
    if (i % 2 == 0) {
      newResponse += responseArray[i]; // Regular text
    } else {
      newResponse += "<code>" + responseArray[i] + "</code>"; // Code formatting for parts inside backticks
    }
  }

  // Replace remaining asterisks with <br> for line breaks
  newResponse = newResponse.split("*").join("</br>");

  return newResponse; // Return the formatted response
}