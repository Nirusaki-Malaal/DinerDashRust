property mgr, name, node_mgr, screen, node, grp, names, scores, rightarrow, leftarrow, menubutton, modetext, modes, currentMode, initialized, save_mgr, game_mgr, submittedRank, enteredMode

on new me, m
  mgr = m
  mm = m.glob.managers
  node_mgr = mm.node
  save_mgr = mm.savegame
  game_mgr = mm.game
  grp = node_mgr.getGroup("titles")
  modes = [#career: "Flo's career", #endless_easy: "endless (easy)", #endless_medium: "endless (medium)", #endless_hard: "endless (hard)"]
  name = #hiscore
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = grp.newNode(getNodeData("hiscore"))
  node.setVisible(0)
  menubutton = newButton([#node: node["menubutton"], #callback: [#target: mgr, #handler: #setMode, #Args: #menu], #members: [#rollover: member("Back_to_Menu_on2", "gfx.screens")]])
  names = []
  scores = []
  repeat with i = 1 to 10
    names.add(node["name_" & i])
    scores.add(node["score_" & i])
  end repeat
  modetext = node["mode"]
  repeat with i in ["leftarrow", "rightarrow"]
    me[symbol(i)] = newButton([#node: node[i], #members: [#rollover: member("hs_arrow_on", "gfx.screens")], #callback: [#target: me, #handler: #clickArrow, #Args: i]])
  end repeat
end

on destroy me
  if initialized then
    menubutton.destroy()
    node.destroy()
  end if
  voidRefs(me)
end

on setPlayMode me, md
  mt = modes[md]
  if voidp(mt) then
    return 
  end if
  currentMode = md
  modetext.setText(mt)
  me.setData(save_mgr.hiscores[md])
  indx = modes.findPos(md)
  leftarrow.setVisible(indx > 1)
  rightarrow.setVisible(indx < modes.count)
end

on clickArrow me, arr
  if voidp(currentMode) then
    currentMode = #career
  end if
  indx = modes.findPos(currentMode)
  if arr = "leftarrow" then
    indx = max(1, min(indx - 1, modes.count))
  else
    if arr = "rightarrow" then
      indx = max(1, min(indx + 1, modes.count))
    end if
  end if
  me.setPlayMode(modes.getPropAt(indx))
end

on setData me, dat
  if voidp(dat) then
    dat = []
  end if
  ct = dat.count
  repeat with i = 1 to names.count
    if i > ct then
      names[i].setText(EMPTY)
      scores[i].setText(EMPTY)
      next repeat
    end if
    d = dat[i]
    n = names[i]
    s = scores[i]
    if (i = submittedRank) and (currentMode = enteredMode) then
      n.sourcemem.color = rgb(92, 255, 255)
      s.sourcemem.color = rgb(92, 255, 255)
    else
      n.sourcemem.color = rgb(255, 255, 255)
      s.sourcemem.color = rgb(255, 255, 255)
    end if
    n.setText(i & ". " & d.name)
    s.setText(string(d.score))
    n.setMember(n.sourcemem)
    s.setMember(s.sourcemem)
  end repeat
end

on getFakeData me
  ls = []
  repeat with i = 1 to 7
    ls.add([#name: "ABC", #score: 10000 - (i * 1000)])
  end repeat
  return ls
end

on enterMode me, previousMode, sr
  submittedRank = sr
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  m = game_mgr.playMode
  if m = #endless then
    lvl = game_mgr.currentDifficulty
    if lvl = 1 then
      m = #endless_easy
    else
      if lvl = 2 then
        m = #endless_medium
      else
        if lvl = 3 then
          m = #endless_hard
        end if
      end if
    end if
  end if
  enteredMode = VOID
  if modes.findPos(m) = 0 then
    m = #career
  else
    enteredMode = m
  end if
  me.setPlayMode(m)
  grp.setVisible(1)
  node.setVisible(1)
end

on event me, sym
end

on leaveMode me, nextMode
  node.disembody()
  if nextMode.name <> #menu then
    grp.setVisible(0)
  end if
  node.setVisible(0)
end
