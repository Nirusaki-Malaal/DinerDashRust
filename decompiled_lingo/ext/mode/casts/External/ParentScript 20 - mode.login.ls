property save_mgr, node_mgr, okbutton, oklocs, slotbuttons, fields, buttons, node, grp, name, selectedSlot, selectMem, playerList, colors, mgr, initialized, editing, timer, cursorChar, editText, textColors, nextMode

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  save_mgr = m.glob.managers.savegame
  grp = node_mgr.getGroup("titles")
  textColors = [#normal: rgb(255, 255, 255), #edit: rgb(255, 255, 128), #wrong: rgb(255, 84, 84), #selected: rgb(128, 255, 255)]
  name = #login
  me.init()
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = grp.newNode(getNodeData("login"))
  node.setVisible(0)
  selectMem = member("login_selected", "gfx.screens")
  buttonmems = [#normal: member("login_normal", "gfx.screens"), #rollover: member("login_rollover", "gfx.screens"), #down: selectMem]
  boxes = []
  oklocs = []
  fields = []
  slotbuttons = []
  cb = [#target: me, #handler: #selectSlot]
  repeat with i = 1 to 10
    oklocs.add(node["ok_" & i].getLoc())
    cb[#Args] = i
    slotbuttons.add(newButton([#node: node["box_" & i], #members: buttonmems, #callback: cb]))
    fields.add(node["text_" & i])
  end repeat
  buttons = [:]
  repeat with i in ["ok", "cancel", "delete", "play"]
    mems = [#normal: member("login_" & i, "gfx.screens"), #rollover: member("login_" & i & "_up", "gfx.screens")]
    buttons[symbol(i)] = newButton([#node: node[string(i)], #members: mems, #callback: [#target: me, #handler: symbol("click" & i), #Args: i]])
  end repeat
  buttons.Ok.setLoc(point(0, 0))
  buttons.Ok.setVisible(0)
  timer = script("timeout").new()
  timer.setPeriod(300)
  timer.setGroup("titles")
  timer.setCallback([#target: me, #handler: #flashCursor])
end

on destroy me
  node.destroy()
  call(#destroy, buttons)
  call(#destroy, slotbuttons)
  timer.destroy()
  voidRefs(me)
end

on enterMode me, previousMode, arg
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  nextMode = arg
  if voidp(nextMode) then
    nextMode = #menu
  end if
  editing = 0
  playerList = Keys(save_mgr.getProfiles())
  selectedSlot = VOID
  node.setVisible(1)
  getNodeGroup("game").setVisible(0)
  me.Refresh()
end

on leaveMode me, nextMode
  node.setVisible(0)
  node.disembody()
end

on Refresh me
  timer.stop()
  buttons.Ok.setVisible(0)
  playerList = Keys(save_mgr.getProfiles())
  ct = playerList.count
  vis = (selectedSlot <= ct) and (selectedSlot > 0)
  buttons.delete.setVisible(vis)
  buttons.play.setVisible(vis)
  repeat with i = 1 to 10
    sb = slotbuttons[i]
    f = fields[i]
    m = f.sourcemem
    m.alignment = #center
    if i = (ct + 1) then
      f.setText("[New Player]")
      m.color = textColors.edit
      sb.setCallback([#target: me, #handler: #editSlot, #Args: i])
      sb.setDisabled(0)
      sb.node.setAlpha(100)
    else
      if i > ct then
        f.setText(EMPTY)
        sb.setDisabled(1)
        sb.setCallback(VOID)
        sb.node.setAlpha(30)
      else
        if i = selectedSlot then
          f.setText(playerList[i])
          m.color = textColors.selected
          sb.setDisabled(1)
          sb.setCallback(VOID)
          sb.node.setMember(selectMem)
          sb.node.setAlpha(100)
        else
          f.setText(playerList[i])
          m.color = textColors.normal
          sb.setCallback([#target: me, #handler: #selectSlot, #Args: i])
          sb.setDisabled(0)
          sb.node.setAlpha(100)
        end if
      end if
    end if
    f.setMember(m)
  end repeat
end

on selectSlot me, num
  if (selectedSlot = num) or (num < 1) or (num > 10) then
    return 
  end if
  selectedSlot = num
  me.Refresh()
end

on flashCursor me
  if not editing then
    timer.stop()
    cursorChar = EMPTY
    return 
  end if
  if cursorChar = EMPTY then
    cursorChar = "__"
  else
    cursorChar = EMPTY
  end if
  fields[selectedSlot].setText(editText & cursorChar)
end

on editSlot me, num
  selectedSlot = num
  me.Refresh()
  sb = slotbuttons[num]
  sb.setDisabled(1)
  sb.node.setMember(selectMem)
  editing = 1
  editText = EMPTY
  cursorChar = "__"
  f = fields[selectedSlot]
  m = f.sourcemem
  m.alignment = #left
  m.color = textColors.edit
  f.setText(editText & cursorChar)
  f.setMember(m)
  buttons.Ok.setLoc(oklocs[selectedSlot])
  buttons.Ok.setVisible(0)
  subscribe(me, #keyDown, "titles")
  buttons.delete.setVisible(0)
  buttons.play.setVisible(0)
  timer.start()
end

on keyDown me, k
  num = charToNum(k)
  f = fields[selectedSlot]
  cc = editText.char.count
  if k = RETURN then
    if me.checkName(editText) then
      me.clickOK()
      return 
    end if
  else
    if num = 8 then
      if cc > 1 then
        editText = editText.char[1..cc - 1]
        cc = cc - 1
      else
        if cc = 1 then
          editText = EMPTY
          cc = 0
        end if
      end if
    else
      if cc > 11 then
        return 
      end if
      if (num < 32) or (num > 126) then
        return 
      end if
      editText = editText & k
    end if
  end if
  f.setText(editText & cursorChar)
  Ok = me.checkName(editText)
  if Ok or (cc = 0) then
    f.sourcemem.color = textColors.edit
  else
    f.sourcemem.color = textColors.wrong
  end if
  buttons.Ok.setVisible(Ok)
end

on checkName me, t
  wc = t.word.count
  if wc = 0 then
    return 
  end if
  s = EMPTY
  repeat with i = 1 to wc
    s = s & t.word[i]
    if i < wc then
      s = s & SPACE
    end if
  end repeat
  repeat with i in playerList
    if i = s then
      return 
    end if
  end repeat
  return 1
end

on clickOK me
  wc = editText.word.count
  if wc = 0 then
    alert("Please type in a name.")
    return 
  end if
  s = EMPTY
  repeat with i = 1 to wc
    s = s & editText.word[i]
    if i < wc then
      s = s & SPACE
    end if
  end repeat
  if playerList.getOne(s) then
    alert("That name is already taken.")
    me.editSlot(selectedSlot)
    return 
  end if
  buttons.Ok.setVisible(0)
  save_mgr.newProfile(s)
  unsubscribe(me, #keyDown, "titles")
  editing = 0
  me.Refresh()
end

on clickCancel me
  mgr.setMode(#menu)
end

on clickPlay me
  if (selectedSlot < 1) or (selectedSlot > playerList.count) then
    return 
  end if
  save_mgr.setCurrentProfile(playerList[selectedSlot])
  mgr.setMode(nextMode)
end

on clickDelete me
  if (selectedSlot < 1) or (selectedSlot > playerList.count) then
    return 
  end if
  if playerList.count < selectedSlot then
    return 
  end if
  save_mgr.deleteProfile(playerList[selectedSlot])
  selectedSlot = 0
  me.Refresh()
end
