property table, sitters, mood, states, state, tableChain, sitterChains, linkedGroups, room_mgr, game_mgr, mood_timer, state_timer, baseTip, moodTip, slowness, patience, minMS, stateData, drinkMood, timeMood, colorMood, actionPoints, leavePenalty, heartsRequired, views, wantedSnack, customerConfig, pickedUp, delay, config, groupType, graphicType, type
global glob

on new me, Args
  views = []
  linkedGroups = []
  sitterChains = []
  room_mgr = Args.room_manager
  game_mgr = Args.game_manager
  customerConfig = glob.managers.config.customer
  type = #sitterGroup
  groupType = Args.type
  tp = glob.managers.config.group.types[groupType]
  if voidp(tp) then
    complain("no groupType of " & groupType & " defined in config.xml")
  end if
  graphicType = glob.managers.config.group.types[groupType].graphicType
  if ["business_female", "old_male", "young_female", "boy", "man", "critic"].getOne(graphicType) = 0 then
    graphicType = "young_female"
  end if
  config = Args.config
  repeat with i = 1 to config.count
    me.setProp(config.getPropAt(i), config[i])
  end repeat
  actionPoints = getConfigData().actionPoints
  delay = Args.delay
  mood_timer = script("mood_timer", "scripts.game").new([#minMS: minMS, #callback: [#target: me, #handler: #changeMood, #Args: -1]])
  sit = Args[#sitters]
  sitters = []
  repeat with i in sit
    sitters.add(script("sitter").new([#sitterGroup: me, #config: config, #color: i.color]))
  end repeat
  mood = 6
  me.setupStates()
  state = states.new
  return me
end

on setSnacks me, ls, pc
  call(#setSnacks, states, ls, pc)
end

on setWantedSnack me, sn
  if sn = wantedSnack then
    return 
  end if
  if not voidp(wantedSnack) then
    ss = room_mgr.snackStations[wantedSnack]
    if not voidp(ss) then
      ss.removeGroup(me)
    end if
  end if
  wantedSnack = sn
  if not voidp(wantedSnack) then
    ss = room_mgr.snackStations[wantedSnack]
    if not voidp(ss) then
      ss.addGroup(me)
    end if
  end if
  table.showBubble(sn)
end

on setPickedUp me, p
  pickedUp = p
end

on addView me, vu
  if views.getOne(vu) = 0 then
    views.add(vu)
  end if
end

on removeView me, vu
  views.deleteOne(vu)
end

on setTimeMultiplier me, num
  call(#setTimeMultiplier, states, num)
end

on destroy me
  mood_timer.destroy()
  mood_timer = VOID
  call(#destroy, states)
  states.deleteAll()
  room_mgr = VOID
  sitterChains.deleteAll()
  tableChain = VOID
  call(#destroy, sitters)
  sitters.deleteAll()
  call(#removeObject, views, me)
  views.deleteAll()
end

on startMoodTimer me, hap, mad
  mood_timer.setHappyMS(hap)
  mood_timer.setMadMS(mad)
  mood_timer.start()
end

on stopMoodTimer me, addresults
  r = mood_timer.stop()
  if addresults then
    me.changeMood(r * timeMood)
  end if
end

on setState me, s
  if state.name = s then
    return 
  end if
  if s = #snack then
    nothing()
  end if
  if (s = #leaving) or (s = #ditching) then
    room_mgr.kitchen.killFood(table)
  end if
  call(#setState, sitters, s)
  st = states[s]
  if not voidp(state) then
    state.leaveState(st)
  end if
  if voidp(st) then
    return 
  end if
  st.enterState(state)
  state = st
end

on getState me
  return state.name
end

on getColorBonus me, num
  me.changeMood(num * colorMood)
end

on changeMood me, num
  m = mood + num
  if pickedUp then
    m = max(1, m)
  end if
  me.setMood(m)
  if not voidp(mood_timer) then
    mood_timer.reset()
  end if
  if num < 0 then
    state.event(#mood, mood)
  end if
  call(#update, sitters)
  call(#update, views)
end

on setMood me, num
  m = min(max(0, num), 10)
  if m < mood then
  else
    if m > mood then
      SndSFX("sfx_get_heart_1")
    end if
  end if
  mood = m
end

on nextState me
  snum = states.getPos(state)
  if (snum + 1) <= states.count then
    me.setState(states.getPropAt(snum + 1))
  end if
end

on setTable me, t
  table = t
end

on doFloAction me, flo
  if voidp(table) then
    return #nothing_to_do
  end if
  return state.event(#flo, flo)
end

on haveDrink me, pts
  call(#drink, sitters)
  me.changeMood(pts)
end

on stopWaiting me
  game_mgr.angryCustomer(leavePenalty)
  call(#showPenalty, views, leavePenalty)
  game_mgr.removeWaitingGroup(me)
  me.destroy()
end

on sitAtTable me, t, ch, indx
  if not voidp(table) then
    complain("tried to seat an already seated group")
    return 
  else
    if not voidp(t.sitterGroup) then
      complain("tried to seat a group at an occupied table")
      return 
    end if
  end if
  tbl = t.seatGroup(me, ch, indx)
  if voidp(tbl) then
    return 
  end if
  me.setTable(tbl)
  game_mgr.removeWaitingGroup(me)
  me.setState(#deciding)
end

on leaveTable me
  if voidp(table) then
    return 
  end if
  room_mgr.groupLeft(me, table)
  table.groupLeave(me)
  call(#setChair, sitters, VOID)
  call(#updateChains, linkedGroups)
end

on leaveWithoutPaying me
  game_mgr.angryCustomer(leavePenalty)
  call(#showPenalty, table.views, leavePenalty)
  me.setState(#ditching)
end

on getLinkedGroups me
  return linkedGroups
end

on leaveTip me
  tip = me.calculateTip()
  if groupType = "critic" then
    if mood >= heartsRequired then
      game_mgr.addStar()
      tip[#extra_star] = 1
    end if
  end if
  SndSFX("sfx_bring_check_1")
  table.setTip(tip)
end

on calculateTip me
  total = (sitters.count * baseTip) + (mood * moodTip)
  Info = [:]
  Info = [#total: total]
  return Info
end

on setupStates me
  states = [:]
  k = Keys(stateData)
  k.add("new")
  repeat with i in k
    m = member("sgstate." & i, "scripts.game")
    if m.number = -1 then
      complain("unimplemented group state " & i & " specified in config.xml file")
      next repeat
    end if
    states[i] = m.script.new(me, stateData[i])
  end repeat
end

on setStateTimeMultipliers me, pl
  repeat with k in Keys(pl)
    s = states[k]
    if voidp(s) then
      next repeat
    end if
    s.setStateTimeMultiplier(pl[k])
  end repeat
end
