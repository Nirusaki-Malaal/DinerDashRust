property sg, name, timer, events, timeMultiplier, stateTimeMultiplier, durationMS, minMS, availableSnacks, snackPercent, snackMS

on new me, g, dat
  sg = g
  durationMS = dat.durationMS
  name = #deciding
  availableSnacks = []
  timeMultiplier = 1.0
  stateTimeMultiplier = 1.0
  events = []
  minMS = dat.minMS
  return me
end

on setTimeMultiplier me, num
  timeMultiplier = num
end

on setStateTimeMultiplier me, num
  stateTimeMultiplier = num
end

on setSnacks me, ls, pc
  if voidp(ls) then
    ls = []
  end if
  availableSnacks = ls
  snackPercent = integer(pc)
end

on destroy me
  me.destroyTimer()
  sg = VOID
end

on enterState me, previousState
  me.destroyTimer()
  if previousState.name = #snack then
    timer = timedCall(max(minMS, (durationMS * timeMultiplier * stateTimeMultiplier) - snackMS), [#target: sg, #handler: #setState, #Args: #ready])
  else
    if (random(100) < snackPercent) and availableSnacks.count and ((durationMS * timeMultiplier * stateTimeMultiplier) >= 2000) then
      jitter = durationMS - 2000
      snackMS = durationMS - jitter + random(jitter)
      timer = timedCall(snackMS, [#target: sg, #handler: #setState, #Args: #snack])
    else
      timer = timedCall(max(minMS, durationMS * timeMultiplier * stateTimeMultiplier), [#target: sg, #handler: #setState, #Args: #ready])
    end if
  end if
  if not voidp(sg.table) then
    sg.table.showMenus()
  end if
end

on destroyTimer me
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
end

on event me, sym, arg
  if sym = #flo then
    if arg.hasPayload(#drinks) then
      arg.dropOff(#drinks, sg.table)
      return #Ok
    end if
    return #not_ready
  end if
end

on leaveState me, nextState
  if nextState.name = #snack then
  else
    SndSFX("sfx_drop_menu_1")
  end if
  me.destroyTimer()
end
