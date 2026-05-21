# SPDX-FileCopyrightText: © 2026 Isaac Freund
# SPDX-License-Identifier: 0BSD

(import wayland)
(import xkbcommon)

(def interfaces
  (wayland/scan
    :custom-protocols ["protocol/river-window-management-v1.xml"
                       "protocol/river-xkb-bindings-v1.xml"]))

(def globals @{})
(def required-globals
  @{"river_window_manager_v1" 4
    "river_xkb_bindings_v1" 1})

(def xkb-bindings
  [[:space {:mod4 true} :spawn-foot]
   [:q {:mod4 true} :close]
   [:n {:mod4 true} :focus-next]
   [:Escape {:mod4 true} :exit]])

(def pointer-bindings
  [[:left {:mod4 :true} :move]
   [:right {:mod4 :true} :resize]])

(def wm @{:outputs @[]
          :seats @[]
          # Windows are kept in rendering order.
          # The last window in the array is rendered on top.
          :windows @[]})

(defn output/maybe-destroy [output]
  (if (output :removed)
    (:destroy (output :obj))
    output))

(defn output/create [obj]
  (def output @{:obj obj})
  (defn output/handle-event [event]
    (match event
      [:removed] (put output :removed true)))
  (:set-user-data obj output)
  (:set-handler obj output/handle-event)
  output)

(defn window/set-position [window x y]
  (:set-position (window :node) x y)
  (put window :x x)
  (put window :y y))

(defn window/create [obj]
  (def window @{:obj obj
                :node (:get-node obj)
                :new true})
  (defn window/handle-event [event]
    (match event
      [:closed] (put window :closed true)
      [:dimensions w h] (do (put window :w w) (put window :h h))
      [:pointer-move-requested seat] (put window :pointer-move-requested
                                          {:seat (:get-user-data seat)})
      [:pointer-resize-requested seat edges] (put window :pointer-resize-requested
                                                  {:seat (:get-user-data seat)
                                                   :edges edges})))
  (:set-handler obj window/handle-event)
  (:set-user-data obj window)
  window)

(defn pointer-binding/create [seat button mods action]
  # From /usr/include/linux/input-event-codes.h
  (def button-code {:left 0x110
                    :right 0x111})
  (def binding @{:obj (:get-pointer-binding (seat :obj) (button-code button) mods)})
  (defn handle-event [event]
    (match event
      [:pressed] (put seat :pending-action action)))
  (:set-handler (binding :obj) handle-event)
  (:enable (binding :obj))
  (array/push (seat :pointer-bindings) binding))

(defn xkb-binding/create [seat keysym mods action]
  (def binding @{:obj (:get-xkb-binding (globals "river_xkb_bindings_v1")
                                        (seat :obj) (xkbcommon/keysym keysym) mods)})
  (defn handle-event [event]
    (match event
      [:pressed] (put seat :pending-action action)))
  (:set-handler (binding :obj) handle-event)
  (:enable (binding :obj))
  (array/push (seat :xkb-bindings) binding))

(defn seat/focus [seat window]
  (defn focus-window [window]
    (unless (= (seat :focused) window)
      (:focus-window (seat :obj) (window :obj))
      (put seat :focused window)
      (array/remove (wm :windows) (index-of window (wm :windows)))
      (array/push (wm :windows) window)
      (:place-top (window :node))))
  (cond
    # If there is an explict target window, focus it.
    window (focus-window window)
    # If there is no explict target, focus the window on top if any.
    (def window (last (wm :windows))) (focus-window window)
    # Otherwise clear focus
    (when (seat :focused)
      (:clear-focus (seat :obj))
      (put seat :focused nil))))

(defn seat/pointer-move [seat window]
  (unless (seat :op)
    (seat/focus seat window)
    (:op-start-pointer (seat :obj))
    (put seat :op @{:type :move
                    :window window
                    :start-x (window :x) :start-y (window :y)
                    :dx 0 :dy 0})))

(defn seat/pointer-resize [seat window edges]
  (unless (seat :op)
    (seat/focus seat window)
    (:inform-resize-start (window :obj))
    (:op-start-pointer (seat :obj))
    (put seat :op @{:type :resize
                    :window window
                    :edges edges
                    :start-x (window :x) :start-y (window :y)
                    :start-w (window :w) :start-h (window :h)
                    :dx 0 :dy 0})))

(defn window/maybe-destroy [window]
  (if (window :closed)
    (do
      (:destroy (window :obj))
      (:destroy (window :node)))
    window))

(defn window/manage [window]
  (when (window :new)
    (put window :new nil)
    (window/set-position window 0 0)
    (:propose-dimensions (window :obj) 0 0))

  (when-let [move (window :pointer-move-requested)]
    (put window :pointer-move-requested nil)
    (seat/pointer-move (move :seat) window))

  (when-let [resize (window :pointer-resize-requested)]
    (put window :pointer-resize-requested nil)
    (seat/pointer-resize (resize :seat) window (resize :edges))))


(defn seat/maybe-destroy [seat]
  (if (seat :removed)
    (do
      (each binding (seat :xkb-bindings)
        (:destroy (binding :obj)))
      (each binding (seat :pointer-bindings)
        (:destroy (binding :obj)))
      (:destroy (seat :obj)))
    seat))

(defn seat/action [seat action]
  (case action
    :spawn-foot (ev/spawn (os/proc-wait (os/spawn ["foot"] :p)))
    :close (when-let [window (seat :focused)]
             (:close (window :obj)))
    :focus-next (seat/focus seat (first (wm :windows)))
    :move (when-let [window (seat :hovered)]
            (seat/pointer-move seat window))
    :resize (when-let [window (seat :hovered)]
              (seat/pointer-resize seat window {:bottom true :right true}))
    :exit (:exit-session (globals "river_window_manager_v1"))))

(defn seat/manage [seat]
  (when (seat :new)
    (put seat :new nil)
    (each binding xkb-bindings
      (xkb-binding/create seat ;binding))
    (each binding pointer-bindings
      (pointer-binding/create seat ;binding)))

  (when-let [window (seat :focused)]
    (when (window :closed)
      (put seat :focused nil)))

  # If no window was interacted with in the current manage sequence,
  # intentionally pass nil to ensure the window on top has focus.
  # This is necessary to handle new windows for example.
  (seat/focus seat (seat :interacted))
  (put seat :interacted nil)

  (when-let [action (seat :pending-action)]
    (put seat :pending-action nil)
    (seat/action seat action))

  (when-let [op (seat :op)
             window (op :window)]
    (cond
      (window :closed)
      (do
        (:op-end (seat :obj))
        (put seat :op nil))

      (seat :op-release)
      (do
        (when (= :resize (op :type))
          (:inform-resize-end (window :obj)))
        (:op-end (seat :obj))
        (put seat :op nil))

      (= :resize (op :type))
      (let [w (max 1 (cond
                       ((op :edges) :left) (- (op :start-w) (op :dx))
                       ((op :edges) :right) (+ (op :start-w) (op :dx))
                       (op :start-w)))
            h (max 1 (cond
                       ((op :edges) :top) (- (op :start-h) (op :dy))
                       ((op :edges) :bottom) (+ (op :start-h) (op :dy))
                       (op :start-h)))]
        (:propose-dimensions (window :obj) w h))))
  (put seat :op-release nil))

(defn seat/render [seat]
  (when-let [op (seat :op)
             window (op :window)]
    (case (op :type)
      :move (window/set-position
              window
              (+ (op :start-x) (op :dx))
              (+ (op :start-y) (op :dy)))
      :resize (window/set-position
                window
                (if ((op :edges) :left)
                  (+ (op :start-x) (- (op :start-w) (window :w)))
                  (op :start-x))
                (if ((op :edges) :top)
                  (+ (op :start-y) (- (op :start-h) (window :h)))
                  (op :start-y))))))


(defn seat/create [obj]
  (def seat @{:obj obj
              :new true
              :xkb-bindings @[]
              :pointer-bindings @[]})
  (defn seat/handle-event [event]
    (match event
      [:removed] (put seat :removed true)
      [:pointer-enter window] (put seat :hovered (:get-user-data window))
      [:pointer-leave] (put seat :hovered nil)
      [:window-interaction window] (put seat :interacted (:get-user-data window))
      [:op-delta dx dy] (do (put (seat :op) :dx dx) (put (seat :op) :dy dy))
      [:op-release] (put seat :op-release true)))
  (:set-handler obj seat/handle-event)
  (:set-user-data obj seat)
  seat)

(defn wm/manage []
  (update wm :outputs |(keep output/maybe-destroy $))
  (update wm :windows |(keep window/maybe-destroy $))
  (update wm :seats |(keep seat/maybe-destroy $))

  (map window/manage (wm :windows))
  (map seat/manage (wm :seats))

  (:manage-finish (globals "river_window_manager_v1")))

(defn wm/render []
  (map seat/render (wm :seats))
  (:render-finish (globals "river_window_manager_v1")))

(defn wm/handle-event [event]
  (match event
    [:unavailable] (do
                     (print "another window manager is already running")
                     (os/exit 1))
    [:finished] (os/exit 0)
    [:manage-start] (wm/manage)
    [:render-start] (wm/render)
    [:output obj] (array/push (wm :outputs) (output/create obj))
    [:seat obj] (array/push (wm :seats) (seat/create obj))
    [:window obj] (array/push (wm :windows) (window/create obj))))


(defn main [& args]
  (def display (wayland/connect interfaces))

  # Avoid passing WAYLAND_DEBUG on to our children.
  # It only matters if it's set when the display is created.
  (os/setenv "WAYLAND_DEBUG" nil)

  (def registry (:get-registry display))
  (:set-handler
    registry
    (defn registry/handle-event [event]
      (match event
        [:global name interface version]
        (when-let [required-version (get required-globals interface)]
          (when (< version required-version)
            (errorf "wayland compositor supported %s version too old (need %d, got %d)"
                    interface required-version version))
          (put globals interface (:bind registry name interface required-version))))))

  (:roundtrip display)
  (eachk i required-globals
    (unless (get globals i)
      (errorf "wayland compositor does not support %s" i)))

  (:set-handler (globals "river_window_manager_v1") wm/handle-event)

  (forever (:dispatch display)))
