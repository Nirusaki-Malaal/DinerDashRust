property tipinfo, bonuses, node, text, timer, extrastar, states, state

on new me, n
  node = n
  text = node["total"]
  states = [:]
  states[#done] = [#handler: #done]
  states[#base] = [#handler: #showBase, #nextState: #done, #time: 2000]
  states[#base_hide] = [#handler: #hide, #nextState: #mood, #time: 500]
  states[#mood] = [#handler: #showMood, #nextState: #mood_hide, #time: 2000]
  states[#mood_hide] = [#handler: #hide, #nextState: #chain, #time: 500]
  states[#chain] = [#handler: #showChain, #nextState: #done, #time: 2000]
  extrastar = node["extrastar"]
  state = states.done
  return me
end

on setState me, sym
  s = states[sym]
  if voidp(s) or (s = state) then
    return 
  end if
  state = s
  if not voidp(s[#handler]) then
    call(s.handler, me, s[#Args])
  end if
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
  if not voidp(s[#nextState]) then
    t = integer(s[#time])
    timer = timedCall(t, [#target: me, #handler: #setState, #Args: s.nextState])
  end if
end

on nextState me
  if not voidp(state[#next]) then
    me.setState(state.next)
  end if
end

on showBase me
  if voidp(tipinfo) then
    return 
  end if
  text.setText("$" & string(tipinfo.total))
  if not voidp(extrastar) then
    extrastar.setVisible(integer(tipinfo[#extra_star]))
  end if
  node.setVisible(1)
end

on showMood me
  if voidp(bonuses) then
    return 
  end if
  if voidp(bonuses[#mood]) then
    me.nextState()
    return 
  end if
  node.setVsisible(1)
  text.setText("mood bonus = " & string(bonuses.mood))
end

on showChain me
  if voidp(bonuses) then
    return 
  end if
  if voidp(bonuses[#tableChain]) then
    me.nextState()
    return 
  end if
  node.setVisible(1)
  tc = bonuses.tableChain
  text.setText("chain x" & string(tc[1]) & " = " & string(tc[3]))
end

on done me
  me.hide()
  tipinfo = VOID
  bonuses = VOID
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
end

on hide me
  node.setVisible(0)
end

on showTip me, tp
  tipinfo = tp
  me.setState(#base)
end

on destroy me
  node = VOID
  text = VOID
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
end
