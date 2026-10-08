property sg, name, events, timer, timeMultiplier, durationMS, minMS

on new me, s, dat
  sg = s
  durationMS = 1500
  name = #leaving
  timeMultiplier = 1.0
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

on enterState me, s
  if durationMS then
    ms = durationMS
    if not voidp(timer) then
      timer.destroy()
      timer = VOID
    end if
    timer = timedCall(ms, [#target: me, #handler: #leave])
  else
    sg.leaveTable()
  end if
end

on leave me
  sg.leaveTable()
end

on event me, sym, arg
  if sym = #leave then
    sg.leaveTable()
  else
    if sym = #flo then
      if sg.table = #dirty then
        t = sg.table
        if arg.pickUp(#dishes) and t.dirty then
          t.clearDishes()
          return #Ok
        end if
        return #hands_full
      end if
    end if
  end if
end

on leaveState me, s
end
