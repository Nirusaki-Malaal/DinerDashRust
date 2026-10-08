property tableobj, chair_nodes, chair_nums, chair_views, table_nodes, sitter_nodes, legnodes, node, anchorNode, capacity, type, sitterviews, dragGroup, mouselistener, plates, drinks, menus, menus_closed, foods, tipnode, dirty, dirty_nodes, hilite, check, checkmarks, numSelects, sitterGroup, lastindex, tableNode, heartrow, tablenum, tablenum_bg, tablenum_hilite, destroyNode, zOrderList, flipList, view_mgr, bar, bonusloc, bubble, partnode, settings
global glob

on new me, Args
  view_mgr = glob.managers.view
  if Args[#isBar] then
    type = #bar
    bar = Args.bar
  else
    capacity = Args.capacity
    if capacity = 2 then
      type = "2top"
      positions = ["left", "right"]
      zOrderList = [1, 2]
      flipList = [1]
    else
      if capacity = 4 then
        type = "4top"
        positions = ["topleft", "topright", "bottomright", "bottomleft"]
        zOrderList = [1, 2, 3, 4]
        flipList = [1, 4]
      else
        if capacity = 6 then
          type = "6top"
          positions = ["left3", "right3", "right2", "right1", "left1", "left2"]
          zOrderList = [1, 2, 3, 6, 4, 5]
          flipList = [1, 5, 6]
        end if
      end if
    end if
  end if
  node = Args.node
  dragGroup = getDragGroup("game.tables")
  sitterviews = []
  chair_views = []
  settings = []
  chair_nodes = []
  chair_nums = []
  sitter_nodes = []
  legnodes = []
  plates = []
  menus_closed = []
  foods = []
  table_nodes = []
  mouselistener = newMouseListener([#target: me])
  destroyNode = Args[#destroyNode]
  if destroyNode then
    node = Args.node
    node.setVisible(0)
    node.setLoc(point(-5000, -5000))
    chair_mouserects = []
    repeat with i = 1 to capacity
      stringi = string(i)
      chair_nodes.add(node["chair_" & stringi])
      chair_nums.add(node["chairnum_" & stringi])
      sitter_nodes.add(node["sitter_" & stringi])
      legnodes.add(node["leg_" & stringi])
      s = node["setting_" & stringi]
      settings.add(s)
      plates.add(s["plate"])
      menus_closed.add(s["menu_closed"])
      foods.add(s["food"])
      s.setVisible(1)
      call(#setVisible, s.children, 0)
      mt = node["mousetile_" & stringi]
      chair_mouserects.add(mt)
      mouselistener.addNode(mt)
    end repeat
    dirty = node["dirty"]
    tableNode = script("fakegroup_node").new(chair_mouserects)
    partnode = node
    tn = node["table_1"]
    table_nodes.add(tn)
    c = chair_nodes[1]
    node.setRegPoint(c.getLoc())
  else
    node = Args.node
    capacity = Args.settings.count
    positions = []
    repeat with setting in Args.settings
      settings.add(setting)
      positions.add("bar")
      plates.add(setting["plate"])
      menus_closed.add(setting["menu_closed"])
      foods.add(setting["food"])
      setting.setVisible(1)
      call(#setVisible, setting.children, 0)
    end repeat
    repeat with i in [#table_nodes, #chair_nodes, #sitter_nodes, #chair_nums, #legnodes, #dirty_nodes]
      me[i] = Args[i]
    end repeat
    chair_mouserects = table_nodes
    partnode = Args.tablestuff
    tableNode = script("fakegroup_node").new(table_nodes)
    dirty = script("fakegroup_node").new(dirty_nodes)
    ch = chair_nodes[integer(float(capacity) / 2.0)]
    partnode["bubble"].setLocRel(ch.getLocRel())
  end if
  repeat with i = 1 to capacity
    chair_views.add(script("chair_view").new([#chair: chair_nodes[i], #number: chair_nums[i], #tableNode: tableNode, #dragGroup: dragGroup, #pos: positions[i], #mouserect: chair_mouserects[i]]))
    sitterviews.add(script("sitter_view").new([#node: sitter_nodes[i], #legnode: legnodes[i], #table_view: me]))
  end repeat
  dirty.setVisible(0)
  bubble = partnode["bubble"]
  bubble.setVisible(0)
  bubble.setZOffset(500)
  tipnode = partnode["tip"]
  tipnode.setVisible(0)
  heartrow = script("heartrow_view", "scripts.game").new(partnode["heartrow"])
  bonusloc = partnode["bonusText"]
  tablenum = partnode["tablenum"]
  tablenum_bg = partnode["tablenum_bg"]
  hilite = partnode["tablenum_hilite"]
  hilite.setVisible(0)
  check = partnode["check"]
  checkmarks = []
  repeat with i = 1 to 3
    checkmarks.add(partnode["checkmark_" & i])
  end repeat
  call(#setZOffset, checkmarks, 1000000)
  call(#moveToFront, checkmarks)
  me.setSelected(0)
  me.setHilite(0)
  check.setVisible(0)
  if not voidp(tableobj) then
    me.setObject(tableobj)
  end if
  return me
end

on setRestaurant me, rname
  if type = #bar then
    return 
  end if
  node["table_1"].setMember(member("table_" & type, "gfx." & rname))
  call(#setRestaurant, chair_views, rname)
end

on showBonus me, Args
  view_mgr.showTableBonus(bonusloc, Args)
end

on showPenalty me, Args
  view_mgr.showPenalty(bonusloc, Args)
end

on setObject me, t
  if not voidp(tableobj) then
    tableobj.removeView(me)
  end if
  tableobj = t
  if voidp(t) then
    me.hideAll()
    node.setVisible(0)
    node.setLoc(point(-5000, -5000))
    return 
  end if
  repeat with i = 1 to chair_views.count
    chair_views[i].setObject(tableobj.chairs[i])
  end repeat
  me.setSelected(0)
  me.setHilite(0)
  me.align()
  me.hideNumber()
end

on removeObject me, tobj
  if tableobj = tobj then
    me.setObject(VOID)
  end if
end

on mouseEnter me
  if not voidp(tableobj) then
    tableobj.mouseover()
  end if
end

on mouseLeave me
  if not voidp(tableobj) then
    tableobj.mouseOut()
  end if
end

on mouseDown me
  if not voidp(tableobj) then
    tableobj.clickOn()
  end if
end

on mouseUp me
  if not voidp(tableobj) then
    tableobj.mouseUp()
  end if
end

on update me, isNewGroup
  if not voidp(tableobj.sitterGroup) then
    sitterGroup = tableobj.sitterGroup
  end if
  if voidp(sitterGroup) then
    call(#setSitter, sitterviews, VOID)
    heartrow.setObject(VOID)
    return 
  end if
  if isNewGroup then
    heartrow.setObject(sitterGroup)
  end if
  sitters = sitterGroup.sitters
  ct = sitters.count
  if ct > capacity then
    complain("group doesn't fit at table")
    return 
  end if
  if voidp(lastindex) then
    lastindex = 1
  end if
  if type = #bar then
    lastindex = max(1, min(lastindex, capacity - ct + 1))
    ls = sitterviews
    repeat with i = 1 to sitters.count
      indx = lastindex + i - 1
      sit = sitters[i]
      sv = sitterviews[i]
      if isNewGroup then
        sv.setSitter(sit)
      end if
      cn = chair_nodes[indx]
      sv.snapTo(cn, 0, 1, isNewGroup)
    end repeat
    bar.zsortSitters()
  else
    ls = []
    repeat with i = 1 to capacity
      indx = ((lastindex + i - 2) mod capacity) + 1
      cn = chair_nodes[indx]
      sv = sitterviews[i]
      ls[indx] = sv
      if i <= ct then
        sit = sitters[i]
        sv.setSitter(sit)
      else
        if type <> #bar then
          sv.setSitter(VOID)
        end if
      end if
      if type <> #bar then
        sv.snapTo(cn, 0, flipList.getOne(indx) <> 0, isNewGroup)
        next repeat
      end if
      sv.snapTo(cn, 0, 1, isNewGroup)
    end repeat
  end if
  if type <> #bar then
    if capacity = 4 then
      ls[1].node.putBehind(ls[4].node)
      ls[2].node.putBehind(ls[3].node)
    else
      if capacity = 6 then
        ls[6].node.putBehind(ls[5].node)
        ls[1].node.putBehind(ls[6].node)
        ls[3].node.putBehind(ls[4].node)
        ls[2].node.putBehind(ls[3].node)
      else
        ls[1].node.putBehind(ls[2].node)
      end if
    end if
    node.setZOffset(0)
    node.updateLocZ()
  end if
  if type <> #bar then
    repeat with i = 1 to capacity
      n = (i mod capacity) + 1
      sv = ls[n]
      if not voidp(sv) then
        bubble.setLoc(sv.node.getLoc())
        exit repeat
      end if
    end repeat
  end if
  call(#update, chair_views)
end

on setFull me
  if type <> #bar then
    call(#hideFlag, chair_views)
    return 
  end if
  repeat with i in table_nodes
    mouselistener.addNode(i)
  end repeat
  me.update(1)
end

on setEmpty me
  if type <> #bar then
    return 
  end if
  repeat with i in table_nodes
    mouselistener.removeNode(i)
  end repeat
end

on placeSitters me, sitgroup, indx
  if indx > capacity then
    nothing()
  end if
  isNewGroup = sitterGroup <> sitgroup
  lastindex = indx
  sitterGroup = sitgroup
  me.update(isNewGroup)
end

on setFoodPaused me, p
  call(#setAnimPaused, foods, p)
end

on align me
  if not destroyNode then
    return 
  end if
  t = tableobj.chairs[1].Tile
  if voidp(t) then
    node.setVisible(0)
    return 
  end if
  anchor = t.node
  if voidp(anchor) then
    return 
  end if
  anchor.addChild(node)
  node.moveToBack()
  node.updateLocZ()
  node.setLoc(anchor.getRegPoint())
  node.setVisible(1)
end

on clean me
  me.hidePlates()
  me.hideFood()
end

on setHilite me, h
  hilite.setVisible(h)
end

on setSelected me, s
  if s then
    numSelects = min(3, numSelects + 1)
    checkmarks[numSelects].setVisible(1)
  else
    numSelects = 0
    call(#setVisible, checkmarks, 0)
  end if
end

on showNumber me
  m = member("big_tablenum_" & tableobj.tablenum)
  if m.number <> -1 then
    tablenum.setMember(m)
    tablenum.setVisible(1)
  else
    nothing()
  end if
  tablenum_bg.setVisible(1)
end

on hideNumber me
  tablenum.setVisible(0)
  hilite.setVisible(0)
  tablenum_bg.setVisible(0)
end

on closeMenus me
  call(#setVisible, menus_closed, 0)
  repeat with i = 1 to capacity
    menus_closed[i].setVisible(not voidp(tableobj.chairs[i].sitter))
  end repeat
  call(#setZOffset, settings, 1)
end

on hideMenus me
  call(#setVisible, menus_closed, 0)
end

on showDirty me
  dirty.setVisible(1)
end

on hideDirty me
  dirty.setVisible(0)
end

on showBubble me, tp
  bubble["bubble"].setAnimation("bubble", [#frameMS: 500])
  bubble["food"].setAnimation(string(tp))
  bubble.moveToFront()
  bubble.setVisible(1)
end

on hideBubble me
  bubble["bubble"].animator.stop()
  bubble["food"].animator.stop()
  bubble.setVisible(0)
end

on showPlates me
end

on showFood me, dur
  st = glob.managers.game.currentStage
  if st = 5 then
    foodtype = random(4) - 1
  else
    foodtype = st - 1
  end if
  repeat with i = 1 to capacity
    f = foods[i]
    vis = not voidp(tableobj.chairs[i].sitter)
    f.setVisible(vis)
    if vis then
      num = (3 * foodtype) + random(3)
      z = f.z
      f.setDummyID("food_" & num)
      f.setAnimation("eating", [#frameMS: integer(float(dur) / 4.0)])
      f.setZ(z)
    end if
  end repeat
end

on hideFood me
  call(#setAnimation, foods, "empty")
end

on hidePlates me
  call(#setVisible, foods, 0)
end

on showTip me, ti
  view_mgr.showTip(tipnode, ti)
end

on hideTip me
  if voidp(tipnode) then
    return 
  end if
  tipnode.setVisible(0)
end

on showCheck me
  check.setVisible(1)
end

on hideCheck me
  check.setVisible(0)
end

on hideAll me
  me.hideFood()
  me.hideMenus()
  me.hidePlates()
  me.hideTip()
  me.hideDirty()
  heartrow.setObject(VOID)
end

on showChain me, ch
end

on destroy me
  mouselistener.destroy()
  heartrow.destroy()
  call(#destroy, chair_views)
  call(#destroy, sitterviews)
  if destroyNode then
    node.destroy()
  else
    partnode.destroy()
    tableNode.destroy()
    dirty.destroy()
  end if
  voidRefs(me)
end
