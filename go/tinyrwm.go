package main

import (
	"context"
	"errors"
	"fmt"
	"log"
	"os"
	"os/exec"
	"slices"

	"codeberg.org/river/tinyrwm/go/internal/proto"
	"hazelnut.eclair.cafe/wlcl"
)

type Output struct {
	proto.RiverOutputV1Stub

	ID proto.RiverOutputV1

	Removed bool
}

func (o *Output) HandleRiverOutputV1Removed(ctx context.Context) {
	o.Removed = true
}

func (o *Output) MaybeDestroy() bool {
	if !o.Removed {
		return false
	}

	o.ID.Destroy()
	return true
}

func NewOutput(id proto.RiverOutputV1) *Output {
	output := &Output{
		ID: id,
	}

	id.SetUserData(output)
	return output
}

type Window struct {
	proto.RiverWindowV1Stub

	ID   proto.RiverWindowV1
	Node proto.RiverNodeV1

	X, Y          int32
	Width, Height int32

	New    bool
	Closed bool

	PointerMoveRequested *Seat

	PointerResizeRequested      *Seat
	PointerResizeRequestedEdges uint32
}

func (w *Window) SetPosition(x, y int32) {
	w.Node.SetPosition(x, y)
	w.X, w.Y = x, y
}

func (w *Window) HandleRiverWindowV1Closed(ctx context.Context) {
	w.Closed = true
}

func (w *Window) HandleRiverWindowV1Dimensions(ctx context.Context, width int32, height int32) {
	w.Width, w.Height = width, height
}

func (w *Window) HandleRiverWindowV1PointerMoveRequested(ctx context.Context, seat proto.RiverSeatV1) {
	w.PointerMoveRequested = seat.UserData().(*Seat)
}

func (w *Window) HandleRiverWindowV1PointerResizeRequested(ctx context.Context, seat proto.RiverSeatV1, edges uint32) {
	w.PointerResizeRequested = seat.UserData().(*Seat)
	w.PointerResizeRequestedEdges = edges
}

func (w *Window) MaybeDestroy() bool {
	if !w.Closed {
		return false
	}

	w.Node.Destroy()
	w.ID.Destroy()
	return true
}

func (w *Window) Manage() {
	if w.New {
		w.New = false
		w.SetPosition(0, 0)
		w.ID.ProposeDimensions(0, 0)
	}

	if w.PointerMoveRequested != nil {
		w.PointerMoveRequested.PointerMove(w)
		w.PointerMoveRequested = nil
	}

	if w.PointerResizeRequested != nil {
		w.PointerResizeRequested.PointerResize(w, w.PointerResizeRequestedEdges)
		w.PointerResizeRequested = nil
	}
}

func NewWindow(id proto.RiverWindowV1) *Window {
	window := &Window{
		ID:   id,
		Node: id.GetNode(),
		New:  true,
	}

	id.SetUserData(window)
	return window
}

type SeatOp interface {
	Manage(state *SeatOpState)
	Render(state *SeatOpState)
	InformStart(w *Window)
	InformEnd(w *Window)
}

type SeatOpState struct {
	Window *Window
	Dx, Dy int32
}

type Seat struct {
	proto.RiverSeatV1Stub

	WM *WindowManager

	ID              proto.RiverSeatV1
	XkbBindings     []*XkbBinding
	PointerBindings []*PointerBinding

	Focused    *Window
	Hovered    *Window
	Interacted *Window

	PendingAction func()

	Op         SeatOp
	OpState    SeatOpState
	OpReleased bool

	New     bool
	Removed bool
}

func (s *Seat) Focus(w *Window) {
	wm := s.WM

	if w == nil && len(wm.Windows) > 0 {
		w = wm.Windows[len(wm.Windows)-1]
	}

	if s.Focused == w {
		return
	}

	if w != nil {
		s.ID.FocusWindow(w.ID)

		// Raise to top
		idx := slices.Index(wm.Windows, w)
		wm.Windows = append(wm.Windows[:idx], wm.Windows[idx+1:]...)
		wm.Windows = append(wm.Windows, w)

		w.Node.PlaceTop()
	} else {
		s.ID.ClearFocus()
	}

	s.Focused = w
}

func (s *Seat) HandleRiverSeatV1OpDelta(ctx context.Context, dx int32, dy int32) {
	s.OpState.Dx, s.OpState.Dy = dx, dy
}

func (s *Seat) HandleRiverSeatV1OpRelease(ctx context.Context) {
	s.OpReleased = true
}

func (s *Seat) HandleRiverSeatV1PointerEnter(ctx context.Context, window proto.RiverWindowV1) {
	s.Hovered = window.UserData().(*Window)
}

func (s *Seat) HandleRiverSeatV1PointerLeave(ctx context.Context) {
	s.Hovered = nil
}

func (s *Seat) HandleRiverSeatV1Removed(ctx context.Context) {
	s.Removed = true
}

func (s *Seat) HandleRiverSeatV1WindowInteraction(ctx context.Context, window proto.RiverWindowV1) {
	s.Interacted = window.UserData().(*Window)
}

func (s *Seat) MaybeDestroy() bool {
	if !s.Removed {
		return false
	}

	for _, b := range s.XkbBindings {
		b.ID.Destroy()
	}
	for _, b := range s.PointerBindings {
		b.ID.Destroy()
	}

	s.ID.Destroy()
	return true
}

func (s *Seat) Manage() {
	if s.New {
		s.New = false
		for _, b := range s.XkbBindings {
			b.ID.Enable()
		}
		for _, b := range s.PointerBindings {
			b.ID.Enable()
		}
	}

	if w := s.Focused; w != nil && w.Closed {
		s.Focused = nil
	}

	// If no window was interacted with in the current manage sequence,
	// intentionally pass nil to ensure the window on top has focus.
	// This is necessary to handle new windows for example.
	s.Focus(s.Interacted)
	s.Interacted = nil

	if s.PendingAction != nil {
		s.PendingAction()
		s.PendingAction = nil
	}

	if op := s.Op; op != nil {
		switch {
		case s.OpReleased:
			op.InformEnd(s.OpState.Window)
			fallthrough
		case s.OpState.Window.Closed:
			s.ID.OpEnd()
			s.Op = nil
		default:
			op.Manage(&s.OpState)
		}
	}
	s.OpReleased = false
}

func (s *Seat) Render() {
	if op := s.Op; op != nil {
		op.Render(&s.OpState)
	}
}

func (s *Seat) StartOp(w *Window, op SeatOp) {
	if s.Op != nil {
		return
	}

	s.Op = op
	s.OpState = SeatOpState{
		Window: w,
	}

	s.Focus(w)

	s.ID.OpStartPointer()
	op.InformStart(w)
}

func (s *Seat) PointerMove(w *Window) {
	s.StartOp(w, NewSeatOpMove(w))
}

func (s *Seat) PointerResize(w *Window, edges uint32) {
	s.StartOp(w, NewSeatOpResize(w, edges))
}

func NewSeat(id proto.RiverSeatV1, wm *WindowManager) *Seat {
	const (
		// See xkbcommon-keysyms.h
		spaceSym = 0x0020
		nSym     = 0x006e
		qSym     = 0x0071
		escSym   = 0xff1b

		leftButton  = 0x110
		rightButton = 0x111
	)

	const super = proto.RiverSeatV1ModifiersMod4

	seat := &Seat{
		WM:  wm,
		ID:  id,
		New: true,
	}

	seat.XkbBindings = []*XkbBinding{
		NewXkbBinding(seat, spaceSym, super, func() {
			cmd := exec.Command("foot")
			if err := cmd.Start(); err != nil {
				log.Printf("Failed to launch foot: %v", err)
			} else {
				cmd.Process.Release()
			}
		}),
		NewXkbBinding(seat, nSym, super, func() {
			if len(seat.WM.Windows) > 0 {
				seat.Focus(seat.WM.Windows[0])
			}
		}),
		NewXkbBinding(seat, qSym, super, func() {
			if seat.Focused != nil {
				seat.Focused.ID.Close()
			}
		}),
		NewXkbBinding(seat, escSym, super, func() {
			seat.WM.WindowManagerV1.ExitSession()
		}),
	}

	seat.PointerBindings = []*PointerBinding{
		NewPointerBinding(seat, leftButton, super, func() {
			if seat.Hovered != nil {
				seat.PointerMove(seat.Hovered)
			}
		}),
		NewPointerBinding(seat, rightButton, super, func() {
			if seat.Hovered != nil {
				seat.PointerResize(seat.Hovered, proto.RiverWindowV1EdgesRight|proto.RiverWindowV1EdgesBottom)
			}
		}),
	}

	seat.ID.SetUserData(seat)
	return seat
}

type SeatOpMove struct {
	StartX, StartY int32
}

func (s *SeatOpMove) Manage(state *SeatOpState) {}

func (s *SeatOpMove) Render(state *SeatOpState) {
	state.Window.SetPosition(s.StartX+state.Dx, s.StartY+state.Dy)
}

func (s *SeatOpMove) InformStart(w *Window) {}

func (s *SeatOpMove) InformEnd(w *Window) {}

func NewSeatOpMove(w *Window) *SeatOpMove {
	return &SeatOpMove{
		StartX: w.X,
		StartY: w.Y,
	}
}

type SeatOpResize struct {
	StartX, StartY          int32
	StartWidth, StartHeight int32
	Edges                   uint32
}

func (s *SeatOpResize) Manage(state *SeatOpState) {
	w, h := s.StartWidth, s.StartHeight
	switch {
	case s.Edges&proto.RiverWindowV1EdgesLeft != 0:
		w -= state.Dx
	case s.Edges&proto.RiverWindowV1EdgesRight != 0:
		w += state.Dx
	}

	switch {
	case s.Edges&proto.RiverWindowV1EdgesTop != 0:
		h -= state.Dy
	case s.Edges&proto.RiverWindowV1EdgesBottom != 0:
		h += state.Dy
	}

	state.Window.ID.ProposeDimensions(max(w, 1), max(h, 1))
}

func (s *SeatOpResize) Render(state *SeatOpState) {
	x := s.StartX
	if s.Edges&proto.RiverWindowV1EdgesLeft != 0 {
		x += s.StartWidth - state.Window.Width
	}

	y := s.StartY
	if s.Edges&proto.RiverWindowV1EdgesTop != 0 {
		y += s.StartHeight - state.Window.Height
	}

	state.Window.SetPosition(x, y)
}

func (s *SeatOpResize) InformStart(w *Window) {
	w.ID.InformResizeStart()
}

func (s *SeatOpResize) InformEnd(w *Window) {
	w.ID.InformResizeEnd()
}

func NewSeatOpResize(w *Window, edges uint32) *SeatOpResize {
	return &SeatOpResize{
		StartX:      w.X,
		StartY:      w.Y,
		StartWidth:  w.Width,
		StartHeight: w.Height,
		Edges:       edges,
	}
}

type XkbBinding struct {
	proto.RiverXkbBindingV1Stub

	ID proto.RiverXkbBindingV1

	Seat     *Seat
	ActionFn func()
}

func (p *XkbBinding) HandleRiverXkbBindingV1Pressed(ctx context.Context) {
	p.Seat.PendingAction = p.ActionFn
}

func NewXkbBinding(seat *Seat, keysym uint32, mods uint32, fn func()) *XkbBinding {
	b := &XkbBinding{
		ID:       seat.WM.XkbBindingsV1.GetXkbBinding(seat.ID, keysym, mods),
		Seat:     seat,
		ActionFn: fn,
	}

	b.ID.SetUserData(b)
	return b
}

type PointerBinding struct {
	proto.RiverPointerBindingV1Stub

	ID proto.RiverPointerBindingV1

	Seat     *Seat
	ActionFn func()
}

func (p *PointerBinding) HandleRiverPointerBindingV1Pressed(ctx context.Context) {
	p.Seat.PendingAction = p.ActionFn
}

func NewPointerBinding(seat *Seat, button uint32, mods uint32, fn func()) *PointerBinding {
	b := &PointerBinding{
		ID:       seat.ID.GetPointerBinding(button, mods),
		Seat:     seat,
		ActionFn: fn,
	}

	b.ID.SetUserData(b)
	return b
}

type WindowManager struct {
	proto.WlRegistryStub
	proto.RiverWindowManagerV1Stub

	Registry        proto.WlRegistry
	WindowManagerV1 proto.RiverWindowManagerV1
	XkbBindingsV1   proto.RiverXkbBindingsV1

	Done bool
	Err  error

	Outputs []*Output
	Windows []*Window
	Seats   []*Seat
}

func (wm *WindowManager) HandleWlRegistryGlobal(ctx context.Context, name uint32, iface string, version uint32) {
	switch iface {
	case proto.RiverWindowManagerV1Name:
		if version >= 4 {
			wm.WindowManagerV1 = proto.As[proto.RiverWindowManagerV1](wm.Registry.Bind(name, iface, 4))
		}
	case proto.RiverXkbBindingsV1Name:
		wm.XkbBindingsV1 = proto.As[proto.RiverXkbBindingsV1](wm.Registry.Bind(name, iface, 1))
	}
}

func (wm *WindowManager) HandleRiverWindowManagerV1Unavailable(ctx context.Context) {
	wm.Done = true
	wm.Err = errors.New("another window manager is already running")
}

func (wm *WindowManager) HandleRiverWindowManagerV1Finished(ctx context.Context) {
	wm.Done = true
}

func (wm *WindowManager) HandleRiverWindowManagerV1Output(ctx context.Context, id proto.RiverOutputV1) {
	wm.Outputs = append(wm.Outputs, NewOutput(id))
}

func (wm *WindowManager) HandleRiverWindowManagerV1Window(ctx context.Context, id proto.RiverWindowV1) {
	wm.Windows = append(wm.Windows, NewWindow(id))
}

func (wm *WindowManager) HandleRiverWindowManagerV1Seat(ctx context.Context, id proto.RiverSeatV1) {
	wm.Seats = append(wm.Seats, NewSeat(id, wm))
}

func (wm *WindowManager) HandleRiverWindowManagerV1ManageStart(ctx context.Context) {
	wm.Outputs = slices.DeleteFunc(wm.Outputs, (*Output).MaybeDestroy)
	wm.Windows = slices.DeleteFunc(wm.Windows, (*Window).MaybeDestroy)
	wm.Seats = slices.DeleteFunc(wm.Seats, (*Seat).MaybeDestroy)

	for _, w := range wm.Windows {
		w.Manage()
	}
	for _, s := range wm.Seats {
		s.Manage()
	}

	wm.WindowManagerV1.ManageFinish()
}

func (wm *WindowManager) HandleRiverWindowManagerV1RenderStart(ctx context.Context) {
	for _, s := range wm.Seats {
		s.Render()
	}

	wm.WindowManagerV1.RenderFinish()
}

func run(ctx context.Context) (err error) {
	conn, err := wlcl.Connect(ctx, "")
	if err != nil {
		return fmt.Errorf("connect: %w", err)
	}
	defer func() {
		if cerr := conn.Close(); cerr != nil && err == nil {
			err = cerr
		}
	}()

	// Don't pass WAYLAND_DEBUG on to children, the added noise makes
	// debugging the window manager itself impractical.
	// It only matters if it's set when the display is created.
	os.Unsetenv("WAYLAND_DEBUG")

	display := proto.As[proto.WlDisplay](conn.Init(proto.Interfaces))

	wm := &WindowManager{}

	wm.Registry = display.GetRegistry()
	wm.Registry.SetUserData(wm)

	if err := wlcl.Roundtrip(ctx, display); err != nil {
		return fmt.Errorf("roundtrip: %w", err)
	}

	if !wm.WindowManagerV1.IsSet() || !wm.XkbBindingsV1.IsSet() {
		return fmt.Errorf("no required globals")
	}

	wm.WindowManagerV1.SetUserData(wm)

	for !wm.Done {
		if err := conn.Dispatch(ctx); err != nil {
			return fmt.Errorf("dispatch: %w", err)
		}
	}

	return wm.Err
}

func main() {
	ctx := context.Background()
	if err := run(ctx); err != nil {
		log.Fatal(err)
	}
}
