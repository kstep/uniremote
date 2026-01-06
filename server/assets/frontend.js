// Extract and store auth token from URL
let authToken = null;

function extractAuthToken() {
    const urlParams = new URLSearchParams(window.location.search);
    const token = urlParams.get('token');
    if (token) {
        authToken = token;
        // Store in sessionStorage for persistence across page navigations
        sessionStorage.setItem('authToken', token);
        window.location.search = '';
        window.history.replaceState({}, document.title, window.location.pathname);
    } else {
        // Try to retrieve from sessionStorage
        authToken = sessionStorage.getItem('authToken');
    }
}

// Notification system
function createNotificationContainer() {
    let container = document.getElementById('notification-container');
    if (!container) {
        container = document.createElement('div');
        container.id = 'notification-container';
        container.className = 'notification-container';
        document.body.appendChild(container);
    }
    return container;
}

function showNotification(title, message, duration = 5000) {
    const container = createNotificationContainer();
    
    const notification = document.createElement('div');
    notification.className = 'notification';
    
    const content = document.createElement('div');
    content.className = 'notification-content';
    
    const titleEl = document.createElement('div');
    titleEl.className = 'notification-title';
    titleEl.textContent = title;
    
    const messageEl = document.createElement('div');
    messageEl.className = 'notification-message';
    messageEl.textContent = message;
    
    content.appendChild(titleEl);
    content.appendChild(messageEl);
    
    const closeBtn = document.createElement('button');
    closeBtn.className = 'notification-close';
    closeBtn.innerHTML = '&times;';
    closeBtn.setAttribute('aria-label', 'Close notification');
    
    notification.appendChild(content);
    notification.appendChild(closeBtn);
    
    container.appendChild(notification);
    
    const removeNotification = () => {
        notification.classList.add('hiding');
        setTimeout(() => {
            notification.remove();
        }, 300);
    };
    
    closeBtn.addEventListener('click', removeNotification);
    
    if (duration > 0) {
        setTimeout(removeNotification, duration);
    }
}

// Extract remote ID from current URL path (/r/:id)
function getRemoteId() {
    const match = window.location.pathname.match(/^\/r\/([^\/]+)/);
    return match ? match[1] : null;
}

// Main API call function
async function callRemoteAction(action, args = []) {
    const remoteId = getRemoteId();
    if (!remoteId) {
        console.error('No remote ID found in URL');
        showNotification('Error', 'No remote ID found in URL');
        return;
    }

    if (!authToken) {
        console.error('No auth token available');
        showNotification('Authentication Error', 'No authentication token available. Please scan the QR code again.');
        return;
    }

    try {
        const response = await fetch(`/api/r/${remoteId}/call`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`,
            },
            body: JSON.stringify({
                action,
                args,
            }),
        });

        if (!response.ok) {
            let errorMessage = `${response.status} ${response.statusText}`;
            let errorTitle = 'Action Failed';
            
            // Customize error messages based on status code
            switch (response.status) {
                case 401:
                    errorTitle = 'Authentication Error';
                    errorMessage = 'Invalid or expired authentication token. Please scan the QR code again.';
                    break;
                case 404:
                    errorTitle = 'Not Found';
                    errorMessage = 'Remote or action not found.';
                    break;
                case 500:
                    errorTitle = 'Server Error';
                    errorMessage = 'An internal server error occurred. Please try again.';
                    break;
            }
            
            // Try to get more specific error message from response (if available)
            try {
                const errorData = await response.json();
                if (errorData.message) {
                    errorMessage = errorData.message;
                }
            } catch (e) {
                // If JSON parsing fails, use the default message from switch
            }
            
            console.error(`API call failed: ${response.status} ${response.statusText}`);
            showNotification(errorTitle, errorMessage);
        }
    } catch (error) {
        console.error('API call error:', error);
        showNotification('Network Error', 'Failed to connect to the server. Please check your connection and try again.');
    }
}

// Event type callbacks
const eventActions = {
    ontap: (element, action) => {
        element.addEventListener('click', (e) => {
            e.preventDefault();
            callRemoteAction(action);
        });
    },

    onhold: (element, action) => {
        let pressTimer;
        element.addEventListener('mousedown', (e) => {
            e.preventDefault();
            pressTimer = setTimeout(() => {
                callRemoteAction(action);
            }, 500);
        });
        element.addEventListener('mouseup', () => clearTimeout(pressTimer));
        element.addEventListener('mouseleave', () => clearTimeout(pressTimer));
    },

    ondown: (element, action) => {
        element.addEventListener('mousedown', (e) => {
            e.preventDefault();
            callRemoteAction(action);
        });
    },

    onup: (element, action) => {
        element.addEventListener('mouseup', (e) => {
            e.preventDefault();
            callRemoteAction(action);
        });
    },

    onchange: (element, action) => {
        element.addEventListener('change', (e) => {
            const value = e.target.type === 'checkbox' ? e.target.checked : e.target.value;
            callRemoteAction(action, [value]);
        });
    },

    ondone: (element, action) => {
        element.addEventListener('blur', (e) => {
            callRemoteAction(action, [e.target.value]);
        });
    },

    ondoubletap: (element, action) => {
        element.addEventListener('dblclick', (e) => {
            e.preventDefault();
            callRemoteAction(action);
        });
    },

    ontouchsize: (element, action) => {
        // Placeholder for touch size handling
        console.warn('ontouchsize not fully implemented');
    },

    ontouchstart: (element, action) => {
        element.addEventListener('touchstart', (e) => {
            const touch = e.touches[0];
            callRemoteAction(action, [0, touch.clientX, touch.clientY]);
        });
    },

    ontouchend: (element, action) => {
        element.addEventListener('touchend', (e) => {
            callRemoteAction(action);
        });
    },

    ontouchdelta: (element, action) => {
        let startX, startY;
        element.addEventListener('touchstart', (e) => {
            const touch = e.touches[0];
            startX = touch.clientX;
            startY = touch.clientY;
        });
        element.addEventListener('touchmove', (e) => {
            const touch = e.touches[0];
            const deltaX = touch.clientX - startX;
            const deltaY = touch.clientY - startY;
            callRemoteAction(action, [0, deltaX, deltaY]);
            startX = touch.clientX;
            startY = touch.clientY;
        });
    },

    onmultitap: (element, action) => {
        let tapCount = 0;
        let tapTimer;
        element.addEventListener('click', (e) => {
            e.preventDefault();
            tapCount++;
            clearTimeout(tapTimer);
            tapTimer = setTimeout(() => {
                callRemoteAction(action, [tapCount]);
                tapCount = 0;
            }, 300);
        });
    },

    onlaunch: (element, action) => {
        // Call immediately when page loads
        callRemoteAction(action);
    },

    onvolumedown: (element, action) => {
        // Register global key action for volume down
        document.addEventListener('keydown', (e) => {
            if (e.key === 'AudioVolumeDown') {
                e.preventDefault();
                callRemoteAction(action);
            }
        });
    },

    onvolumeup: (element, action) => {
        // Register global key action for volume up
        document.addEventListener('keydown', (e) => {
            if (e.key === 'AudioVolumeUp') {
                e.preventDefault();
                callRemoteAction(action);
            }
        });
    },
};

// Scan DOM and attach event actions
function initializeRemote() {
    // Find all elements with data-on* attributes
    const elements = document.querySelectorAll('[data-ontap], [data-onhold], [data-ondown], [data-onup], [data-onchange], [data-ondone], [data-ondoubletap], [data-ontouchsize], [data-ontouchstart], [data-ontouchend], [data-ontouchdelta], [data-onmultitap], [data-onlaunch], [data-onvolumedown], [data-onvolumeup]');

    elements.forEach(element => {
        // Iterate through all possible event types
        Object.keys(eventActions).forEach(eventType => {
            const action = element.getAttribute(`data-${eventType}`);
            if (action) {
                eventActions[eventType](element, action);
            }
        });
    });
}

// SSE Connection for server updates
let eventSource = null;
let sseAbortController = null;

async function connectSSE() {
    const remoteId = getRemoteId();
    if (!remoteId || !authToken) {
        return;
    }

    // Close existing connection if any
    if (sseAbortController) {
        sseAbortController.abort();
    }

    sseAbortController = new AbortController();
    const sseUrl = `/api/r/${remoteId}/events`;

    try {
        const response = await fetch(sseUrl, {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Accept': 'text/event-stream',
            },
            signal: sseAbortController.signal,
        });

        if (!response.ok) {
            throw new Error(`SSE connection failed: ${response.status}`);
        }

        console.log('SSE connection established');

        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = '';

        while (true) {
            const { done, value } = await reader.read();
            
            if (done) {
                console.log('SSE connection closed by server');
                break;
            }

            buffer += decoder.decode(value, { stream: true });
            const lines = buffer.split('\n');
            buffer = lines.pop(); // Keep incomplete line in buffer

            for (const line of lines) {
                if (line.startsWith('data: ')) {
                    const data = line.substring(6);
                    try {
                        const message = JSON.parse(data);
                        console.log('Received SSE message:', message);
                        handleSSEMessage(message);
                    } catch (error) {
                        console.error('Failed to parse SSE message:', error, data);
                    }
                }
            }
        }
    } catch (error) {
        if (error.name === 'AbortError') {
            console.log('SSE connection aborted');
            return;
        }
        
        console.error('SSE connection error:', error);
        
        // Reconnect after a delay
        setTimeout(() => {
            console.log('Attempting to reconnect SSE...');
            connectSSE();
        }, 5000);
    }
}

// Handle SSE messages from server
function handleSSEMessage(message) {
    const { action, args } = message;

    if (action === 'update' && args && args.id) {
        const element = document.getElementById(args.id);
        if (!element) {
            console.warn(`Widget with id "${args.id}" not found`);
            return;
        }

        // Update element properties based on args
        if (args.text !== undefined) {
            updateElementText(element, args.text);
        }

        if (args.value !== undefined) {
            updateElementValue(element, args.value);
        }

        if (args.checked !== undefined) {
            updateElementChecked(element, args.checked);
        }

        if (args.visibility !== undefined) {
            updateElementVisibility(element, args.visibility);
        }

        if (args.progress !== undefined) {
            updateElementProgress(element, args.progress);
        }
    }
}

// Helper function to update text content
function updateElementText(element, text) {
    const textElement = element.querySelector('.widget-text') || element;
    textElement.textContent = text;
}

// Helper function to update element value
function updateElementValue(element, value) {
    if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA') {
        element.value = value;
    } else if (element.tagName === 'PROGRESS') {
        element.value = value;
    }
}

// Helper function to update checked state
function updateElementChecked(element, checked) {
    if (element.tagName === 'INPUT' && element.type === 'checkbox') {
        element.checked = checked;
    }
}

// Helper function to update visibility
function updateElementVisibility(element, visibility) {
    switch (visibility) {
        case 'visible':
            element.style.visibility = 'visible';
            element.style.display = '';
            break;
        case 'invisible':
            element.style.visibility = 'hidden';
            break;
        case 'gone':
            element.style.display = 'none';
            break;
    }
}

// Helper function to update progress/slider value
function updateElementProgress(element, progress) {
    const progressElement = element.querySelector('input[type="range"]');
    if (progressElement) {
        progressElement.value = progress;
    } else if (element.tagName === 'INPUT' && element.type === 'range') {
        element.value = progress;
    }
}

// Initialize the application
function initializeApp() {
    extractAuthToken();
    initializeRemote();
    connectSSE();
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeApp);
} else {
    initializeApp();
}
