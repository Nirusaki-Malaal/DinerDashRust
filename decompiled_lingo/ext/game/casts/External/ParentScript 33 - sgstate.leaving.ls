property sg, name, events, timer, timeMultiplier, stateTimeMultiplier, durationMS, minMS

on new me, s, dat
  sg = s
  durationMS = 1500
  name = #leaving
  timeMultiplier = 1.0
  stateTimeMultiplier = 1.0
  minMS = dat.minMS
  return me
end

on destroy me
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
  sg = VOID
end

on setTimeMultiplier me, num
  timeMultiplier = num
end

on setStateTimeMultiplier me, num
  stateTimeMultiplier = num
end

on enterState me, s
  if durationMS then
    ms = durationMS
    timer = timedCall(ms, [#target: sg, #handler: #leaveTable])
  else
    sg.leaveTable()
  end if
end

on event me, sym, arg
  if sym = #leave then
    sg.leaveTable()
  else
    if sym = #flo then
      t = sg.table
      if t.dirty then
        if arg.pickUp(#dishes) then
          t.clearDishes()
          return #Ok
        end if
      end if
      return #hands_full
    end if
  end if
end

on leaveState me, s
end
