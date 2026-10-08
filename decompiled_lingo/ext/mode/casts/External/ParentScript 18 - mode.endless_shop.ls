property mgr, name, node_mgr, game_mgr, screen, node, grp, inventext, itembuttons, itemdesc, check, timer, continuebutton, initialized, numUpgrades

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  game_mgr = m.glob.managers.game
  grp = node_mgr.getGroup("titles")
  name = #endless_shop
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = grp.newNode(getNodeData("endless_shop"))
  node.setVisible(0)
  node.setZOffset(9999999)
  check = node["check"]
  itemdesc = node["item_desc"]
  itembuttons = []
  itemnames = []
  bscript = script("eshop_button")
  repeat with i = 1 to 3
    itembuttons.add(bscript.new([#icon: node["item_" & i], #name: node["itemname_" & i], #desc: itemdesc, #mgr: me]))
  end repeat
  inventext = [:]
  repeat with i in [#podium, #stereo, #shoes, #oven, #maitred, #drinks]
    inventext[i] = node["text_" & i]
  end repeat
  timer = script("timeout").new()
  timer.oneshot = 1
  timer.setPeriod(500)
  timer.setCallback([#target: me, #handler: #getUpgrades])
  timer.setGroup(#mode)
  continuebutton = newButton([#node: node["continue_button"]])
end

on destroy me
  if initialized then
    call(#destroy, itembuttons)
    timer.destroy()
    node.destroy()
  end if
  voidRefs(me)
end

on selectItem me, inf, nd
  game_mgr.addInventoryItem(inf)
  check.setLoc(nd.getLoc())
  check.setVisible(1)
  call(#setDisabled, itembuttons)
  timer.start()
end

on getUpgrades me
  numUpgrades = numUpgrades - 1
  check.setVisible(0)
  itemdesc.setText(EMPTY)
  buy = game_mgr.getUpgradeSet()
  if voidp(buy) then
    buy = []
  end if
  inven = game_mgr.inventory
  ct = inventext.count
  repeat with i = 1 to ct
    p = inventext.getPropAt(i)
    t = inventext[i]
    inv = inven[p]
    if voidp(inv) then
      t.setVisible(0)
      next repeat
    end if
    nm = inv.name
    if nm = #table then
      next repeat
    end if
    t.setText(inv.name)
    t.setVisible(1)
  end repeat
  ct = buy.count
  repeat with i = 1 to 3
    ib = itembuttons[i]
    if i > ct then
      ib.setItem(VOID)
      next repeat
    end if
    ib.setItem(buy[i])
  end repeat
  if buy.count = 0 then
    continuebutton.setVisible(1)
    continuebutton.setCallback([#target: mgr, #handler: #setMode, #Args: #play])
    return 
  end if
  continuebutton.setVisible(0)
end

on showDone me
end

on enterMode me, previousMode, dat
  numUpgrades = dat.upgrades
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  node["mousetrap"].addMouseListener(me)
  game_mgr.setPaused(1)
  grp.setVisible(1)
  node.setVisible(1)
  me.getUpgrades()
end

on event me, sym
end

on leaveMode me, nextMode
  timer.stop()
  continuebutton.setCallback(VOID)
  node["mousetrap"].removeMouseListener(me)
  game_mgr.setPaused(0)
  game_mgr.upgradeButton.setVisible(0)
  node.setVisible(0)
  node.disembody()
end
