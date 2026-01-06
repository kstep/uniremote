// Extract and store auth token from URL
let authToken = null;
let ws = null;
let wsReconnectTimer = null;
let wsReconnectAttempts = 0;
const WS_MAX_RECONNECT_ATTEMPTS = 5;
const WS_RECONNECT_DELAY = 2000;

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

// Extract remote ID from current URL path (/r/:id)
function getRemoteId() {
    const match = window.location.pathname.match(/^\/r\/([^\/]+)/);
    return match ? match[1] : null;
}

// WebSocket connection management
function connectWebSocket() {
    const remoteId = getRemoteId();
    if (!remoteId || !authToken) {
        console.log('No remote ID or auth token available, skipping WebSocket connection');
        return;
    }

    // Close existing connection if any
    if (ws) {
        ws.close();
        ws = null;
    }

    // Construct WebSocket URL
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/r/${remoteId}/ws`;

    console.log('Connecting to WebSocket:', wsUrl);

    try {
        // Pass auth token via WebSocket subprotocol (bearer.{token})
        ws = new WebSocket(wsUrl, [`bearer.${authToken}`]);

        ws.onopen = () => {
            console.log('WebSocket connected');
            wsReconnectAttempts = 0;
            if (wsReconnectTimer) {
                clearTimeout(wsReconnectTimer);
                wsReconnectTimer = null;
            }
        };

        ws.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                handleServerMessage(message);
            } catch (e) {
                console.error('Failed to parse WebSocket message:', e);
            }
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };

        ws.onclose = (event) => {
            console.log('WebSocket closed:', event.code, event.reason);
            ws = null;

            // Attempt to reconnect with exponential backoff
            if (wsReconnectAttempts < WS_MAX_RECONNECT_ATTEMPTS) {
                wsReconnectAttempts++;
                const delay = WS_RECONNECT_DELAY * Math.pow(2, wsReconnectAttempts - 1);
                console.log(`Reconnecting in ${delay}ms (attempt ${wsReconnectAttempts}/${WS_MAX_RECONNECT_ATTEMPTS})`);
                wsReconnectTimer = setTimeout(connectWebSocket, delay);
            } else {
                console.error('Max WebSocket reconnection attempts reached');
                showNotification('Connection Lost', 'Lost connection to server. Please refresh the page.');
            }
        };
    } catch (e) {
        console.error('Failed to create WebSocket:', e);
    }
}

// Handle incoming messages from server
function handleServerMessage(message) {
    console.log('Received server message:', message);

    switch (message.type) {
        case 'update':
            handleUpdateMessage(message);
            break;
        case 'error':
            showNotification('Error', message.message);
            break;
        default:
            console.warn('Unknown message type:', message.type);
    }
}

// Handle update messages (e.g., {"type":"update","action":"update","args":{"id":"widget-id","text":"new text"}})
function handleUpdateMessage(message) {
    const args = message.args || {};
    
    // If there's an id in args, update that specific element
    if (args.id) {
        const element = document.getElementById(args.id);
        if (element) {
            // Update text content if provided
            if (args.text !== undefined) {
                if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA') {
                    element.value = args.text;
                } else {
                    element.textContent = args.text;
                }
            }
            
            // Update value if provided (for inputs)
            if (args.value !== undefined && (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA')) {
                element.value = args.value;
            }
            
            // Update checked state if provided (for checkboxes)
            if (args.checked !== undefined && element.type === 'checkbox') {
                element.checked = args.checked;
            }
            
            console.log(`Updated element ${args.id}`);
        } else {
            console.warn(`Element with id '${args.id}' not found`);
        }
    }
}

// Main API call function via WebSocket
function callRemoteAction(action, args = []) {
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

    // Use WebSocket if connected, otherwise fall back to HTTP
    if (ws && ws.readyState === WebSocket.OPEN) {
        const message = {
            type: 'call',
            action: action,
            args: (args && args.length > 0) ? args : null
        };
        
        try {
            ws.send(JSON.stringify(message));
            console.log('Sent action via WebSocket:', action, args);
        } catch (e) {
            console.error('Failed to send WebSocket message:', e);
            // Fall back to HTTP
            callRemoteActionHTTP(action, args);
        }
    } else {
        // WebSocket not available, use HTTP fallback
        console.log('WebSocket not connected, using HTTP fallback');
        callRemoteActionHTTP(action, args);
    }
}

// HTTP fallback for calling actions
async function callRemoteActionHTTP(action, args = []) {
    const remoteId = getRemoteId();
    
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

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        extractAuthToken();
        initializeRemote();
        connectWebSocket();
    });
} else {
    extractAuthToken();
    initializeRemote();
    connectWebSocket();
}

// Clean up WebSocket on page unload
window.addEventListener('beforeunload', () => {
    if (ws) {
        ws.close();
    }
    if (wsReconnectTimer) {
        clearTimeout(wsReconnectTimer);
    }
});
