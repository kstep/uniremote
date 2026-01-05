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
        return;
    }

    if (!authToken) {
        console.error('No auth token available');
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
            console.error(`API call failed: ${response.status} ${response.statusText}`);
        }
    } catch (error) {
        console.error('API call error:', error);
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
    });
} else {
    extractAuthToken();
    initializeRemote();
}
