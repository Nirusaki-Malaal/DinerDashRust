property y, z, x, xreg, yreg, xscale, yscale, rect, visible, rotation, xflip, yflip, parentXFlip, parentYFlip, baseXReg, baseYReg, zdirty, alpha, parentAlpha, parentVisible, screenProps, offscreen, sprite_mgr, pSprite, alphamod, xAxis, yAxis, zAxis, parent, localID, stringme, zoffset, type, HIDDEN, baseWidth, baseHeight, mouseListeners, copymem, sourcemem, pMember, dynamic, depth, text, ink, bgColor, embodied, animator
global glob

on new me, sm, alldata
  if voidp(alldata) then
    alldata = [#frames: [[:]]]
  end if
  data = alldata.frames[1]
  type = #leaf
  mouseListeners = []
  if voidp(data) then
    data = [:]
  end if
  stringme = string(me)
  sprite_mgr = sm
  children = []
  namedChildren = [:]
  count = 0
  dynamic = alldata[#dynamic]
  pSprite = sprite_mgr.getSprite()
  mouseListeners = []
  offscreen = point(-5000, -5000)
  parentAlpha = 100
  parentVisible = 1
  visible = 1
  if not voidp(data[#blend]) then
    alpha = data[#blend]
  else
    alpha = 100
  end if
  pSprite.blend = alpha
  pSprite.ink = integer(data[#ink])
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
  zAxis = vector(0, 0, 1)
  rp = data[#regPoint]
  if voidp(rp) then
    rp = point(0.0, 0.0)
  end if
  baseXReg = rp[1]
  baseYReg = rp[2]
  baseWidth = 0
  baseHeight = 0
  width = 0.0
  height = 0.0
  localID = string(alldata[#nodeID])
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
  zflip = (2 * ((xflip or yflip) and not (xflip and yflip))) - 1
  if not voidp(data[#member]) and (data[#cast] <> "gfx.layout") then
    m = member(data.member, data[#cast])
    if m.number <> -1 then
      me.setMember(member(data.member, data.cast))
      if m.type = #text then
        me.setText(m.text)
      end if
    end if
  else
    complain("using leaf node where it should use branch")
  end if
  screenProps = [:]
  repeat with i in [#x, #y, #z, #alpha, #blend, #visible, #xflip, #yflip, #xscale, #yscale, #rotation]
    screenProps[i] = me[i]
  end repeat
  ink = integer(data[#ink])
  pSprite.ink = ink
  if ink = 36 then
    bgColor = data[#bgColor]
    if voidp(bgColor) then
      bgColor = color(0)
    end if
    pSprite.bgColor = bgColor
  end if
  ad = alldata[#animdata]
  if voidp(ad) then
    ad = getAnimData(alldata.frames)
    alldata[#animdata] = ad
  end if
  me.setAnimData(ad)
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
end

on disembody me
  if voidp(pSprite) then
    return 
  end if
  sprite_mgr.returnSprite(pSprite)
  pSprite = VOID
end

on embody me
  if not voidp(pSprite) then
    return 
  end if
  zdirty = 1
  pSprite = sprite_mgr.getSprite()
  if voidp(pSprite) then
    return 
  end if
  if mouseListeners.count then
    pSprite.scriptInstanceList.add(me)
  end if
  if not voidp(pMember) then
    me.setMember(pMember)
  end if
  pSprite.ink = ink
  if ink = 36 then
    pSprite.bgColor = bgColor
  end if
  me.setAlpha(alpha)
  me.setVisible(visible)
  me.updateTransform()
end

on newNode me, layerinfo, tp
  node = script("node2d").new(sprite_mgr, layerinfo)
  me.addChild(node)
  if not voidp(layerinfo) then
    repeat with i in layerinfo.children
      node.newNode(i)
    end repeat
  end if
  return node
end

on setAnimPaused me, p
  if voidp(animator) then
    return 
  end if
  animator.setPaused(p)
end

on setAnimation me, str, Args
  if voidp(animator) then
    return 
  end if
  animator.setAnimation(str, Args)
end

on queueAnimation me, str, Args
  if voidp(animator) then
    return 
  end if
  animator.queueAnimation(str, Args)
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

on updateFlip me
  if voidp(parent) then
    fl = [0, 0]
  else
    fl = parent.getWorldFlip()
  end if
  parentXFlip = fl[1]
  parentYFlip = fl[2]
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

on destroy me
  count = 0
  if not voidp(animator) then
    animator.destroy()
    animator = VOID
  end if
  if not voidp(parent) then
    parent.removeChild(me)
  end if
  if not voidp(copymem) then
    erase(copymem)
  end if
  if voidp(pSprite) then
    return 
  end if
  sprite_mgr.returnSprite(pSprite)
  sprite_mgr = VOID
  voidRefs(me)
end

on setMember me, m
  if m.number = -1 then
    return 
  end if
  if not dynamic then
    sourcemem = m
    pMember = m
    copymem = VOID
  else
    me._setupDynamicMember(m)
  end if
  if not voidp(pSprite) then
    pSprite.member = pMember
  end if
  rp = pMember.regPoint
  baseXReg = rp[1]
  baseYReg = rp[2]
  baseWidth = pMember.width
  baseHeight = pMember.height
  me.updateTransform()
end

on getMember me
  return pSprite.member
end

on setParentFlip me, fl
  parentXFlip = fl[1]
  parentYFlip = fl[2]
  me.updateTransform()
end

on setParentXFlip me, b
  if parentXFlip = b then
    return 
  end if
  parentXFlip = b
  me.updateTransform()
end

on setParentYFlip me, b
  if parentYFlip = b then
    return 
  end if
  parentYFlip = b
  me.updateTransform()
end

on setXFlip me, b
  if xflip = b then
    return 
  end if
  xflip = b
  me.updateTransform()
end

on setYFlip me, b
  if yflip = b then
    return 
  end if
  yflip = b
  me.updateTransform()
end

on getHeight me
  if voidp(pSprite) then
    return 
  end if
  return pSprite.rect.height
end

on setHeight me, h
  height = h
  yscale = height / baseHeight
  if yflip then
    yreg = height - (baseYReg * yscale)
  else
    yreg = baseYReg * yscale
  end if
end

on getWidth me
  if voidp(pSprite) then
    return 
  end if
  return pSprite.rect.width
end

on setWidth me, w
  width = w
  xscale = width / baseWidth
  if xflip then
    xreg = width - (baseXReg * xscale)
  else
    xreg = baseXReg * xscale
  end if
end

on setRect me, r
  if voidp(pSprite) then
    return 
  end if
  rect = r
  width = r.width
  height = r.height
  xscale = width / baseWidth
  yscale = height / baseHeight
  pSprite.rect = r
end

on getRect me
  if voidp(pSprite) then
    return 
  end if
  return pSprite.rect
end

on updateRect me
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
  if num = 1 then
    nothing()
  end if
  z = num
  if not voidp(pSprite) then
    pSprite.locZ = z
  end if
  me.updateTransform()
end

on getZ me
  return z
end

on getNumLayers me
  return 1
end

on setZDirty me
  if zdirty then
    return 
  end if
  zdirty = 1
  if not voidp(parent) then
    parent.setZDirty()
  end if
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
  if voidp(pSprite) then
    return 
  end if
  pSprite.rotation = r
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
  abs_alpha = min(100, max(0, a * parentAlpha * 0.01))
  if voidp(pSprite) then
    return 
  end if
  pSprite.blend = abs_alpha * alphamod
  me.updateTransform()
end

on setParentAlpha me, a
  parentAlpha = a
  me.setAlpha(alpha)
end

on getAlpha me
  return alpha
end

on addMouseListener me, obj
  ct = mouseListeners.count
  if mouseListeners.getOne(obj) = 0 then
    mouseListeners.add(obj)
    if (ct = 0) and not voidp(pSprite) then
      si = pSprite.scriptInstanceList
      if si.getOne(me) = 0 then
        si.add(me)
      end if
    end if
  end if
end

on removeMouseListener me, obj
  if voidp(pSprite) then
    return 
  end if
  if not mouseListeners.deleteOne(obj) then
    return 
  end if
  if mouseListeners.count = 0 then
    pSprite.scriptInstanceList.deleteOne(me)
    return 1
  end if
end

on mouseEnter me
  call(#mouseEnter, mouseListeners)
end

on mouseLeave me
  call(#mouseLeave, mouseListeners)
end

on mouseDown me
  glob.mouseDown()
  call(#mouseDown, mouseListeners)
end

on mouseUp me
  call(#mouseUp, mouseListeners)
end

on mouseUpOutSide me
  call(#mouseUpOutSide, mouseListeners)
end

on mouseWithin me
  if voidp(pSprite) then
    return 
  end if
  return (the mouseLoc).inside(pSprite.rect)
end

on setVisible me, vis
  visible = vis
  v = vis and parentVisible
  if v then
    HIDDEN = 0
    me.updateSprite()
  else
    HIDDEN = 1
    if not voidp(pSprite) then
      pSprite.loc = offscreen
    end if
  end if
  me.updateTransform()
end

on setParentVisible me, vis
  parentVisible = vis
  me.setVisible(visible)
end

on getVisible me
  return visible
end

on updateLocZ me
  if not zdirty then
    return 
  end if
  me.parent.setZDirty()
end

on updateTransform me
  if voidp(parent) then
    return 
  end if
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
  screenProps.x = psp.x + myx
  screenProps.y = psp.y + myY
  me.updateSprite()
end

on printZ me
  put "actual z:" & pSprite.locZ && "loc:" & pSprite.loc && "spritenum:" & pSprite.spriteNum && "local z:" & z && pSprite.member.name
end

on updateSprite me
  if voidp(pSprite) then
    return 
  end if
  if not screenProps.visible then
    pSprite.loc = offscreen
    return 
  end if
  if rotation = 0 then
    pSprite.rect = rect(0, 0, screenProps.xscale * baseWidth, screenProps.yscale * baseHeight)
  end if
  pSprite.flipH = screenProps.xflip
  pSprite.flipV = screenProps.yflip
  pSprite.locH = screenProps.x
  pSprite.locV = screenProps.y
  pSprite.blend = screenProps.alpha
  pSprite.rotation = screenProps.rotation
  if screenProps.z <> pSprite.locZ then
    pSprite.locZ = screenProps.z
  end if
end

on setRegPoint me, pt
  baseXReg = pt[1]
  baseYReg = pt[2]
end

on setXReg me, num
  baseXReg = num
  me.updateTransform()
end

on setYReg me, num
  baseYReg = num
  me.updateTransform()
end

on getRegPoint me
  return point(baseXReg, baseYReg)
end

on getXReg me
  return baseXReg
end

on getYReg me, num
  return baseYReg
end

on setZOffset me, num
  zoffset = num
  me.updateLocZ()
  me.updateTransform()
end

on _setMember me, m
  if sourcemem = m then
    return 
  end if
  if m.type = #text then
    dynamic = 1
  end if
  if not dynamic then
    sourcemem = m
    pMember = m
    copymem = VOID
  else
    me._setupDynamicMember(m)
  end if
  pSprite.member = pMember
  pregpt = pMember.regPoint
  if m.type = #text then
    pSprite.color = m.color
    pSprite.bgColor = m.bgColor
    pSprite.ink = 0
  end if
end

on _setupDynamicMember me, m
  lastmem = sourcemem
  sourcemem = m
  if not voidp(lastmem) then
    d = lastmem.image.depth
    r1 = copymem.rect
    r2 = lastmem.rect
    if (copymem.rect <> lastmem.rect) or (depth <> lastmem.image.depth) then
      me.makeNewMember()
    end if
  else
    me.makeNewMember()
  end if
  i = sourcemem.image
  r = i.rect
  baseWidth = r.width
  baseHeight = r.height
  copymem.image.setAlpha(image(copymem.width, copymem.height, 8))
  copymem.image.copyPixels(i, r, r)
  copymem.regPoint = sourcemem.regPoint
  pMember = copymem
end

on makeNewMember me
  oldmem = copymem
  depth = sourcemem.image.depth
  copymem = new(#bitmap, castLib("dynamic"))
  copymem.image = image(sourcemem.width, sourcemem.height, 32)
  copymem.image.useAlpha = 1
  if not voidp(oldmem) then
    erase(oldmem)
  end if
end

on setText me, t
  if text = t then
    return 
  end if
  text = t
  if sourcemem.type <> #text then
    return 
  end if
  sourcemem.text = t
  if not dynamic then
    return 
  end if
  i = sourcemem.image
  if i.rect <> copymem.rect then
    me.setMember(sourcemem)
  else
    i = sourcemem.image
    r = i.rect
    copymem.image.fill(copymem.rect, sourcemem.bgColor)
    copymem.image.setAlpha(image(copymem.width, copymem.height, 8))
    copymem.image.copyPixels(i, r, r)
  end if
  me.updateSprite()
end

on setColor me, c
  if pMember.type = #text then
    pMember.color = c
  else
    pSprite.color = c
  end if
end
