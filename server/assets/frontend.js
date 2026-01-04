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

    try {
        const response = await fetch(`/api/r/${remoteId}/call`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
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
const eventHandlers = {
    ontap: (element, handler) => {
        element.addEventListener('click', (e) => {
            e.preventDefault();
            callRemoteAction(handler);
        });
    },

    onhold: (element, handler) => {
        let pressTimer;
        element.addEventListener('mousedown', (e) => {
            e.preventDefault();
            pressTimer = setTimeout(() => {
                callRemoteAction(handler);
            }, 500);
        });
        element.addEventListener('mouseup', () => clearTimeout(pressTimer));
        element.addEventListener('mouseleave', () => clearTimeout(pressTimer));
    },

    ondown: (element, handler) => {
        element.addEventListener('mousedown', (e) => {
            e.preventDefault();
            callRemoteAction(handler);
        });
    },

    onup: (element, handler) => {
        element.addEventListener('mouseup', (e) => {
            e.preventDefault();
            callRemoteAction(handler);
        });
    },

    onchange: (element, handler) => {
        element.addEventListener('change', (e) => {
            const value = e.target.type === 'checkbox' ? e.target.checked : e.target.value;
            callRemoteAction(handler, [value]);
        });
    },

    ondone: (element, handler) => {
        element.addEventListener('blur', (e) => {
            callRemoteAction(handler, [e.target.value]);
        });
    },

    ondoubletap: (element, handler) => {
        element.addEventListener('dblclick', (e) => {
            e.preventDefault();
            callRemoteAction(handler);
        });
    },

    ontouchsize: (element, handler) => {
        // Placeholder for touch size handling
        console.warn('ontouchsize not fully implemented');
    },

    ontouchstart: (element, handler) => {
        element.addEventListener('touchstart', (e) => {
            const touch = e.touches[0];
            callRemoteAction(handler, [touch.clientX, touch.clientY]);
        });
    },

    ontouchend: (element, handler) => {
        element.addEventListener('touchend', (e) => {
            callRemoteAction(handler);
        });
    },

    ontouchdelta: (element, handler) => {
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
            callRemoteAction(handler, [deltaX, deltaY]);
            startX = touch.clientX;
            startY = touch.clientY;
        });
    },

    onmultitap: (element, handler) => {
        let tapCount = 0;
        let tapTimer;
        element.addEventListener('click', (e) => {
            e.preventDefault();
            tapCount++;
            clearTimeout(tapTimer);
            tapTimer = setTimeout(() => {
                callRemoteAction(handler, [tapCount]);
                tapCount = 0;
            }, 300);
        });
    },

    onlaunch: (element, handler) => {
        // Call immediately when page loads
        callRemoteAction(handler);
    },

    onvolumedown: (element, handler) => {
        // Register global key handler for volume down
        document.addEventListener('keydown', (e) => {
            if (e.key === 'AudioVolumeDown') {
                e.preventDefault();
                callRemoteAction(handler);
            }
        });
    },

    onvolumeup: (element, handler) => {
        // Register global key handler for volume up
        document.addEventListener('keydown', (e) => {
            if (e.key === 'AudioVolumeUp') {
                e.preventDefault();
                callRemoteAction(handler);
            }
        });
    },
};

// Scan DOM and attach event handlers
function initializeRemote() {
    // Find all elements with data-on* attributes
    const elements = document.querySelectorAll('[data-ontap], [data-onhold], [data-ondown], [data-onup], [data-onchange], [data-ondone], [data-ondoubletap], [data-ontouchsize], [data-ontouchstart], [data-ontouchend], [data-ontouchdelta], [data-onmultitap], [data-onlaunch], [data-onvolumedown], [data-onvolumeup]');

    elements.forEach(element => {
        // Iterate through all possible event types
        Object.keys(eventHandlers).forEach(eventType => {
            const handler = element.getAttribute(`data-${eventType}`);
            if (handler) {
                eventHandlers[eventType](element, handler);
            }
        });
    });
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeRemote);
} else {
    initializeRemote();
}
