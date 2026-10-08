property ROOT, zdirty, sprite_mgr, nodescript, mode, testnode, tn2, testmouse

on new me
  return me
end

on init me
  mode = #new
  if mode = #hardware then
    nodescript = script("node3d")
  else
    nodescript = script("node2d")
  end if
  ROOT = nodescript.new(sprite_mgr, [#nodeID: "world", #type: #Branch, #frames: [[#loc: point(0, 0), #z: 0]], #children: []])
  ROOT.setZDirty()
  ROOT.updateLocZ()
  if mode = #new then
    subscribe(me, #exitFrame, "node")
  end if
end

on exitFrame me
  ROOT.updateLocZ()
end

on newButton me, Args
  if mode = #hardware then
    return script("node3d_button").new(Args)
  else
    return script("node2d_button").new(Args)
  end if
end

on newMouseListener me, Args
  if mode = #hardware then
    return script("node3d_mouselistener").new(Args)
  else
    return script("node2d_mouselistener").new(Args)
  end if
end

on destroy me
  unsubscribe(me, #exitFrame, "node")
  sprite_mgr = VOID
  if not voidp(ROOT) then
    ROOT.destroy()
  end if
  ROOT = VOID
  glob = VOID
end

on setSpriteManager me, sm
  sprite_mgr = sm
end

on getRoot
  return ROOT
end

on getGroup me, str
  c = ROOT[str]
  if not voidp(c) then
    return c
  end if
  c = me.newNode(ROOT, [#nodeID: str])
  ROOT.addChild(c)
  return c
end

on newNode me, par, layerinfo
  node = nodescript.new(sprite_mgr, layerinfo)
  if voidp(par) then
    par = ROOT
  end if
  par.addChild(node)
  if not voidp(layerinfo) then
    if not voidp(layerinfo[#children]) then
      repeat with i in layerinfo.children
        node.addChild(me.newNode(par, i))
      end repeat
    end if
  end if
  return node
end

on moveToFront me, nd
  fr = ROOT[1]
  if fr = nd then
    return 
  end if
  nd.putInFrontOf(fr)
end

on moveToBack me, nd
  bk = ROOT[ROOT.child.count]
  if bk = nd then
    return 
  end if
  nd.putInBackOf(bk)
end

on putInFrontOf me, fr, bk
  ROOT.putInFrontOf(fr, bk)
end

on putInBackOf me, bk, fr
  ROOT.putInBackOf(bk, fr)
end

on testnode me
  testnode = me.newNode(ROOT, getNodeData("room"))
  testnode.setLoc(point(0, 0))
  testnode.setVisible(1)
  testnode["texttext"].setText("hahahaha")
end

on testmouse me
  n = testnode["table1"]
  testmouse = newMouseListener()
  testmouse.addNode(n, 1)
  rot = n.getRotation()
  testmouse.setCallback(#mouseEnter, [#handler: #setAlpha, #target: n, #Args: 50])
  testmouse.setCallback(#mouseLeave, [#handler: #setAlpha, #target: n, #Args: 100])
  testmouse.setCallback(#mouseDown, [#handler: #setRotation, #target: n, #Args: rot + 45])
  testmouse.setCallback(#mouseUp, [#handler: #setRotation, #target: n, #Args: rot])
  testmouse.setCallback(#mouseUpOutSide, [#handler: #setRotation, #target: n, #Args: rot])
end

on cleanupTests me
  if not voidp(testmouse) then
    testmouse.destroy()
  end if
  testmouse = VOID
  testnode.destroy()
  testnode = VOID
end

on testLocs me
  n = testnode["table1"]
  t = testnode["floor"].children[1]
  n.setLocRel(point(0, 0), t)
  t.setLoc(point(0, 0))
  tn2 = me.newNode()
  ROOT.addChild(tn2)
  tn2.updateTransform()
  tn2.addChild(n)
  tn2.addChild(t)
  put EMPTY
end

on testMousePrint me
  put "asfasdfasfd"
end
