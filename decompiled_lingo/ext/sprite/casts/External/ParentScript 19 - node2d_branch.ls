property y, z, x, xscale, yscale, rect, visible, rotation, xflip, yflip, parentXFlip, parentYFlip, screenProps, parentProps, baseXReg, baseYReg, zdirty, alpha, parentAlpha, parentVisible, offscreen, sprite_mgr, pSprite, alphamod, xAxis, yAxis, zAxis, parent, children, namedChildren, localID, count, stringme, zoffset, type, animator, dummyID, dirty, embodied
global glob

on new me, sm, alldata
  type = #Branch
  if voidp(alldata) then
    alldata = [#frames: [[:]]]
  end if
  if voidp(alldata[#frames]) then
    alldata[#frames] = [[:]]
  end if
  localID = string(alldata[#nodeID])
  sprite_mgr = sm
  children = []
  namedChildren = [:]
  mouseListeners = []
  zAxis = vector(0, 0, 1)
  count = 0
  stringme = string(me)
  alpha = 100
  me.setNodeData(alldata)
  screenProps = [:]
  repeat with i in [#x, #y, #z, #alpha, #visible, #xflip, #yflip, #xscale, #yscale, #rotation]
    screenProps[i] = me.getProp(i)
  end repeat
  if localID = "floor" then
    nothing()
  end if
  return me
end

on setAnimData me, ad, ap
  if not voidp(animator) then
    animator.destroy()
    animator = VOID
  end if
  if ad.count then
    animator = script("node_anim").new([#node: me, #data: ad])
  end if
  if ilk(ap, #list) then
    animator.setAnimProps(ap)
  end if
end

on setAnimGroup me, gr
  if not voidp(animator) then
    animator.setGroup(gr)
  end if
  call(#setAnimGroup, children, gr)
end

on disembody me
  if not embodied then
    return 
  end if
  embodied = 0
  call(#disembody, children)
end

on embody me, noupdate
  if embodied then
    return 
  end if
  embodied = 1
  zdirty = 1
  call(#embody, children, 1)
  if not noupdate then
    me.updateLocZ()
  end if
end

on setNodeData me, alldata
  if voidp(alldata) then
    alldata = [#frames: [[:]]]
  end if
  if voidp(alldata[#frames]) then
    alldata[#frames] = [[:]]
  end if
  if alldata.frames = [] then
    alldata[#frames] = [[:]]
  end if
  if localID = "floor" then
    nothing()
  end if
  data = alldata.frames[1]
  offscreen = point(-5000, -5000)
  parentAlpha = 100
  parentVisible = 1
  visible = 1
  if not voidp(data[#blend]) then
    alpha = data[#blend]
  else
    alpha = 100
  end if
  alphamod = 1.0
  z = 0
  zdirty = 1
  if not voidp(data[#scale]) then
    xscale = data.scale[1]
    yscale = data.scale[2]
  else
    xscale = 1.0
    yscale = 1.0
  end if
  rp = data[#regPoint]
  if voidp(rp) then
    rp = point(0.0, 0.0)
  end if
  baseXReg = rp[1]
  baseYReg = rp[2]
  zAxis = vector(0, 0, 1)
  baseWidth = 0
  baseHeight = 0
  width = 0.0
  height = 0.0
  if not voidp(data[#rotation]) then
    rotation = integer(data[#rotation])
  end if
  if not voidp(data[#loc]) then
    x = data.loc[1]
    y = data.loc[2]
  else
    x = 0.0
    y = 0.0
  end if
  xflip = integer(data[#xflip])
  yflip = integer(data[#yflip])
  me.setAnimData(getAnimData(alldata.frames))
  if not voidp(data[#member]) then
    complain("can't assign member to a branch node")
  end if
end

on newNode me, layerinfo, noupdatez
  if not voidp(layerinfo) then
    if layerinfo.children.count or layerinfo[#noSprite] then
      node = script("node2d_branch").new(sprite_mgr, layerinfo)
      me.addChild(node, VOID, VOID, 1)
      repeat with i in layerinfo.children
        node.newNode(i, 1)
      end repeat
    else
      node = script("node2d_leaf").new(sprite_mgr, layerinfo)
      me.addChild(node, VOID, VOID, 1)
    end if
  else
    node = script("node2d_branch").new(sprite_mgr, layerinfo)
    me.addChild(node, VOID, VOID, 1)
  end if
  if noupdatez then
    return node
  end if
  me.updateLocZ()
  return node
end

on addChild me, nd, prev_nd, bfore, noupdatez
  if nd = prev_nd then
    return 
  end if
  if children.getOne(nd) <> 0 then
    children.deleteOne(nd)
  end if
  if voidp(prev_nd) then
    indx = children.count
  else
    indx = children.getOne(prev_nd)
    if indx = 0 then
      indx = children.count
    end if
  end if
  if children.count = 0 then
    children.add(nd)
  else
    children.addAt(indx + not bfore, nd)
  end if
  if nd.localID <> EMPTY then
    namedChildren[nd.localID] = nd
  end if
  count = children.count
  nd.setParent(me)
  me.setZDirty(noupdatez)
  if noupdatez then
    return 
  end if
  me.updateLocZ()
end

on removeChild me, nd
  children.deleteOne(nd)
  namedChildren.deleteOne(nd)
  count = children.count
end

on setParent me, cl
  if cl = parent then
    return 
  end if
  if not voidp(parent) then
    parent.removeChild(me)
    parent = VOID
  end if
  parent = cl
end

on getLocalID me
  return localID
end

on getWorldID me
  return parent.worldID & "." & localID
end

on putInFrontOf me, nd
  if voidp(nd.parent) then
    return 
  end if
  nd.parent.addChild(me, nd)
end

on putBehind me, nd
  if voidp(nd.parent) then
    return 
  end if
  nd.parent.addChild(me, nd, nd, 1)
end

on moveToFront me
  if voidp(parent) then
    return 
  end if
  parent.addChild(me)
  me.setZDirty()
end

on moveToBack me
  if voidp(parent) then
    return 
  end if
  parent.addChild(me, parent.getBackChild(), 1)
  me.setZDirty()
end

on getFrontChild me
  if children.count = 0 then
    return 
  end if
  return children[children.count]
end

on getBackChild me
  if children.count = 0 then
    return 
  end if
  return children[1]
end

on destroy me
  if not voidp(animator) then
    animator.destroy()
    animator = VOID
  end if
  call(#destroy, children.duplicate())
  children.deleteAll()
  namedChildren.deleteAll()
  count = 0
  if not voidp(parent) then
    parent.removeChild(me)
  end if
  sprite_mgr = VOID
end

on setMember me
  complain("can't set member of a branch node")
end

on printZ me
  call(#printZ, children)
end

on setAnimPaused me, p
  call(#setAnimPaused, children, p)
end

on setAnimation me, str, Args
  call(#setAnimation, children, str, Args)
  if voidp(animator) then
    return 
  end if
  animator.setAnimation(str, Args)
end

on queueAnimation me, str, Args
  call(#queueAnimation, children, str, Args)
  if voidp(animator) then
    return 
  end if
  animator.queueAnimation(str, Args)
end

on setDummyID me, str
  if str = dummyID then
    return 
  end if
  dat = getNodeData(str)
  if voidp(dat) then
    return 
  end if
  call(#destroy, children.duplicate())
  children.deleteAll()
  namedChildren.deleteAll()
  me.setNodeData(dat)
  repeat with i in dat.children
    me.newNode(i)
  end repeat
end

on getMember me
  complain("can't get member of a branch node")
end

on setText me
  complain("can't set text of a branch node")
end

on setParentFlip me, fl
  parentXFlip = fl[1]
  parentYFlip = fl[2]
  me.setXFlip(xflip)
  me.setYFlip(yflip)
end

on setParentXFlip me, b
  if parentXFlip = b then
    return 
  end if
  parentXFlip = b
  me.setXFlip(xflip)
end

on setParentYFlip me, b
  if parentYFlip = b then
    return 
  end if
  parentYFlip = b
  me.setYFlip(yflip)
end

on setXFlip me, b
  if xflip = b then
    return 
  end if
  xflip = b
  wf = (parentXFlip or xflip) and not (parentXFlip and xflip)
  call(#setParentXFlip, children, wf)
  me.updateTransform()
end

on setYFlip me, b
  if yflip = b then
    return 
  end if
  yflip = b
  wf = (parentYFlip or yflip) and not (parentYFlip and yflip)
  call(#setParentYFlip, children, wf)
  me.updateTransform()
end

on getHeight me
end

on setHeight me, h
end

on getWidth me
end

on setWidth me, w
end

on setRect me, r
end

on getRect me
  r = rect(x, y, x, y)
  repeat with i in children
    r = union(r, i.getRect())
  end repeat
  return r
end

on setLoc me, pt
  x = pt[1]
  y = pt[2]
  me.updateTransform()
end

on getLoc me
  return point(x, y)
end

on setY me, c
  y = c
  me.updateTransform()
end

on setX me, c
  x = c
  me.updateTransform()
end

on getLocRel me, othernode
  if voidp(othernode) then
    othernode = glob.managers.node.ROOT
  end if
  op = othernode.screenProps
  diff = point(screenProps.x, screenProps.y) - point(op.x, op.y)
  rpdiff = me.getRegPoint() - othernode.getRegPoint()
  return point(diff[1] + rpdiff[1], diff[2] + rpdiff[2])
end

on setLocRel me, pt, othernode
  if voidp(othernode) then
    othernode = glob.managers.node.ROOT
  end if
  me.setLoc(point(x, y) + pt - me.getLocRel(othernode))
end

on getStageLoc me
  if not voidp(parent) then
    par = parent.getStageLoc()
  else
    par = point(0, 0)
  end if
  return par + point(x, y)
end

on setStageLoc me, pt
  me.setLoc(point(x, y) + pt - me.getStageLoc())
end

on setZ me, num
  if (z = num) and not voidp(z) then
    return 
  end if
  z = num
  zdirty = 1
  me.updateLocZ()
  me.updateTransform()
end

on getZ me
  return z
end

on getNumLayers me
  s = 1
  repeat with i in children
    s = s + i.getNumLayers()
  end repeat
  return s
end

on getNumLeaves me
  repeat with i in children
    s = s + (i.type = #leaf)
  end repeat
  return s
end

on updateLocZ me
  if not zdirty then
    return 
  end if
  myz = z
  repeat with i in children
    i.setZ(myz)
    myz = myz + i.getNumLayers()
  end repeat
  zdirty = 0
  me.updateTransform()
end

on setZDirty me
  if zdirty then
    return 
  end if
  zdirty = 1
  if not voidp(parent) then
    parent.setZDirty()
  else
    me.updateLocZ()
  end if
end

on miniprintz me
  repeat with i in children
    put "z:" & i.z && "+ lrs:" & i.getNumLayers() && " = " & i.z + i.getNumLayers()
  end repeat
end

on setScale me, arg1, arg2
  if ilk(arg1, #point) then
    xscale = arg1[1]
    yscale = arg1[2]
  else
    if ilk(arg1, #number) then
      xscale = arg1
      if ilk(arg2, #number) then
        yscale = arg2
      else
        yscale = arg1
      end if
    end if
  end if
  me.updateTransform()
end

on setXScale me, xs
  xscale = xs
  me.updateTransform()
end

on setYScale me, ys
  yscale = ys
  me.updateTransform()
end

on getScale me
  return point(xscale, yscale)
end

on getXScale me
  return xscale
end

on getYScale me
  return yscale
end

on normalizeRotation me
  r = rotation
  repeat while (r < 0) or (r > 359)
    r = (r + 360) mod 360
  end repeat
  if r <> rotation then
    me.setRotation(r)
  end if
end

on setRotation me, r
  rotation = r
  me.updateTransform()
end

on getRotation me
  return rotation
end

on setRotationRel me, r, othernode
  if voidp(othernode) then
    othernode = glob.managers.node.ROOT
  end if
  me.setRotation(rotation + r - me.getRotationRel(othernode))
end

on getRotationRel me, othernode
  if voidp(othernode) then
    othernode = glob.managers.node.ROOT
  end if
  return screenProps.rotation - othernode.screenProps.rotation
end

on setAlpha me, a
  alpha = a
  me.updateTransform()
end

on getAlpha me
  return alpha
end

on addMouseListener me, obj
  call(#addMouseListener, children, obj)
end

on removeMouseListener me, obj
  call(#removeMouseListener, children, obj)
end

on mouseWithin me
  repeat with i in children
    if i.mouseWithin() then
      return 1
    end if
  end repeat
end

on setVisible me, vis
  visible = vis
  call(#setParentVisible, children, vis)
  me.updateTransform()
end

on getVisible me
  return visible
end

on updateTransform me
  if not voidp(parent) then
    psp = parent.screenProps
    screenProps.z = z + zoffset
    screenProps.alpha = min(100, max(0, alpha * psp.alpha * 0.01))
    screenProps.visible = visible and psp.visible
    screenProps.rotation = psp.rotation + rotation
    screenProps.xscale = xscale * psp.xscale
    screenProps.yscale = yscale * psp.yscale
    screenProps.xflip = (parentXFlip or xflip) and not (parentXFlip and xflip)
    screenProps.yflip = (parentYFlip or yflip) and not (parentYFlip and yflip)
    myx = x
    myY = y
    if parentXFlip then
      myx = -x
    end if
    if parentYFlip then
      myY = -y
    end if
    screenProps.x = psp.x + myx - ((1 - (2 * xflip)) * (baseXReg * screenProps.xscale))
    screenProps.y = psp.y + myY - ((1 - (2 * yflip)) * (baseYReg * screenProps.yscale))
  else
    screenProps = me
  end if
  call(#updateTransform, children)
end

on updateSprite me
  call(#updateSprite, children)
end

on setRegPoint me, pt
  baseXReg = pt[1]
  baseYReg = pt[2]
  me.updateTransform()
end

on getRegPoint me
  return point(baseXReg, baseYReg)
end

on setXReg me, num
  baseXReg = num
  me.updateTransform()
end

on getXReg me
  return baseXReg
end

on setYReg me, num
  baseYReg = num
  me.updateTransform()
end

on getYReg me, num
  return baseYReg
end

on setZOffset me, num
  zoffset = num
  me.updateTransform()
  call(#setZOffset, children, num)
end

on getAt me, idstr
  if idstr.integerp then
    if idstr <= children.count then
      return children[idstr]
    end if
    return 
  end if
  return namedChildren[idstr]
end

on getPropAt me, indx
  if (indx > children.count) or not indx.integerp then
    complain("index out of range")
    return 
  end if
  return children[indx].localID
end
