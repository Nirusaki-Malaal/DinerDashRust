global glob

on getlayers Args
  framenum = Args[#framenum]
  absoluteLocs = Args[#absoluteLocs]
  noDummies = Args[#noDummies]
  endFrame = Args[#endFrame]
  Info = [#nodeID: string(framenum), #loc: point(0, 0), #z: 0]
  currentFrame = the frame
  layers = [:]
  if stringp(framenum) then
    framenum = marker(framenum)
  end if
  if stringp(endFrame) then
    endFrame = marker(endFrame)
  end if
  if voidp(endFrame) then
    endFrame = framenum
  end if
  left = 5000
  top = 5000
  repeat with i = 1 to the lastChannel
    go(framenum)
    if sprite(i).memberNum = 0 then
      next repeat
    end if
    s = sprite(i)
    lr = [:]
    lr[#scripts] = s.scriptList.duplicate()
    repeat with xxx in lr[#scripts]
      xxx[2] = value(devoid(xxx[2]))
    end repeat
    lr[#nodeID] = EMPTY
    repeat with scr in lr[#scripts]
      if scr[1] = member("nodeID_b") then
        ls = scr[2]
        lr[#nodeID] = ls[#nodeID]
        did = ls[#dummyID]
        if not (voidp(did) or (did = EMPTY)) then
          lr[#dummyID] = did
        end if
        lr[#noSprite] = ls[#noSprite]
        lr[#dynamic] = ls[#dynamic]
        exclude = ls[#exclude]
        exit repeat
      end if
    end repeat
    if exclude then
      next repeat
    end if
    if not voidp(lr[#dummyID]) and not noDummies then
    end if
    animframes = []
    repeat with f = framenum to endFrame
      go(f)
      frm = [:]
      m = s.member
      if m.number < 1 then
        next repeat
      end if
      scrpts = s.scriptList.duplicate()
      repeat with xxx in scrpts
        xxx[2] = value(devoid(xxx[2]))
      end repeat
      repeat with scr in scrpts
        if scr[1] = member("nodeID_b") then
          ls = scr[2]
          frm[#animID] = ls[#animID]
          frm[#dummyID] = ls[#dummyID]
          exit repeat
        end if
      end repeat
      frm[#regPoint] = m.regPoint
      frm[#rect] = s.rect
      frm[#blend] = s.blend
      frm[#xflip] = s.flipH
      frm[#yflip] = s.flipV
      frm[#rotation] = s.rotation
      if frm.rotation = 0 then
        if m.type = #text then
          iw = m.image.rect.width
          ih = m.image.rect.height
        else
          iw = m.rect.width
          ih = m.rect.height
        end if
        frm[#scale] = point(float(frm.rect.width) / iw, float(frm.rect.height) / ih)
      end if
      frm[#z] = i
      frm[#loc] = sprite(i).loc
      if voidp(lr[#children]) then
        if not lr[#noSprite] then
          frm[#member] = m.name
          frm[#cast] = castLib(m.castLibNum).name
          frm[#ink] = s.ink
          frm[#backColor] = s.backColor
          frm[#foreColor] = s.foreColor
          frm[#bgColor] = s.bgColor
          if m.name = EMPTY then
            alert("sprite" && i && "has member with no name on frame" && framenum)
          end if
        end if
        frm[#regPoint] = m.regPoint
      end if
      animframes.add(frm)
    end repeat
    lr[#frames] = animframes
    if voidp(lr[#children]) then
      lr[#children] = []
    end if
    layers.addProp(lr.nodeID, lr)
    if lr.frames.count then
      fr1 = lr.frames[1]
      left = min(fr1.loc[1] - fr1.regPoint[1], left)
      top = min(fr1.loc[2] - fr1.regPoint[2], top)
    end if
  end repeat
  if not absoluteLocs then
    myloc = point(left, top)
    repeat with j in layers
      repeat with f in j.frames
        f.loc = f.loc - myloc
      end repeat
    end repeat
  end if
  go(currentFrame)
  Info[#children] = layers
  return Info
end

on setupGfxData
  clearGlobals()
  animlist = []
  animtxt = member("anim_list", "scripts.layout").text
  repeat with i = 1 to animtxt.line.count
    animlist.add(animtxt.line[i].word[1])
  end repeat
  txt = member("layout_list", "scripts.layout")
  repeat with i = 1 to txt.line.count
    fr = txt.line[i].word[1]
    put fr
    getSpriteData([#name: fr, #animated: animlist.getOne(fr) > 0])
  end repeat
  makeFlosAnimData()
  makeColors()
end

on getSpriteData Args, nod
  if stringp(Args) then
    nm = Args
    noDummies = nod
  else
    nm = Args.name
    noDummies = Args[#noDummies]
    endfr = VOID
  end if
  if nm.stringp and not marker(nm) then
    return 
  end if
  animlist = []
  animtxt = member("anim_list", "scripts.layout").text
  repeat with i = 1 to animtxt.line.count
    animlist.add(animtxt.line[i].word[1])
  end repeat
  anim = animlist.getOne(nm) > 0
  if anim then
    endfr = nm & "_end"
  end if
  m = member(nm & "_layout", "scripts.layout")
  if m.number = -1 then
    m = new(#text, castLib("scripts.layout"))
    m.name = nm & "_layout"
  end if
  d = getlayers([#framenum: nm, #noDummies: noDummies, #endFrame: endfr])
  m.text = devoid(string(d))
  return d
end

on getNodeData nm
  if not voidp(glob) then
    return glob.getNodeData(nm)
  end if
  m = member(nm & "_layout", "scripts.layout")
  if m.number = -1 then
    return 
  end if
  val = value(devoid(m.text))
  fillDummies(val)
  return val
end

on fillDummies nd
  if voidp(nd) then
    return 
  end if
  if not voidp(nd[#dummyID]) and (nd[#dummyID] <> EMPTY) then
    d = getNodeData(nd.dummyID)
    if not voidp(d) then
      nd.children = d.children
    else
      nothing()
    end if
    nd[#dummyID] = VOID
    nd[#member] = VOID
  end if
  if voidp(nd[#children]) then
    return 
  end if
  repeat with i in nd.children
    fillDummies(i)
  end repeat
end

on makeDummy name
  go(marker("makedummy"))
  sm = script("sprite2d_manager").new()
  cm = script("node_manager").new()
  sm.init()
  cm.setSpriteManager(sm)
  cm.init()
  c = cm.ROOT.newNode(getNodeData(name))
  c.setVisible(1)
  c.setLoc(point(100, 100))
  updateStage()
  r = c.getRect()
  rot = c.getRotation()
  if r[1] < 0 then
    c.setLoc(point(-r[1], c.getLoc()[2]))
  end if
  if r[2] < 0 then
    c.setLoc(point(c.getLoc()[1], -r[2]))
  end if
  c.setRotation(rot)
  updateStage()
  r = c.getRect()
  img = image(r.width, r.height, 32)
  img.copyPixels((the stage).image, img.rect, r)
  c.destroy()
  cm.destroy()
  sm.destroy()
  m = member(name & "_dummy", "gfx.layout")
  if m.number = -1 then
    m = new(#bitmap, castLib("gfx.layout"))
    m.trimWhiteSpace = 0
    m.name = name & "_dummy"
  end if
  m.image = img
  m.regPoint = point(0, 0)
  return m
end

on getAnimData framelist
  anims = [:]
  repeat with i in framelist
    animID = i[#animID]
    if voidp(animID) then
      next repeat
    end if
    anim = anims[animID]
    if voidp(anim) then
      anim = []
      anims[animID] = anim
    end if
    if string(i[#dummyID]) = EMPTY then
      if not voidp(i[#member]) then
        m = member(i.member, i.cast)
        i[#member] = m
      end if
    end if
    anim.add(i)
  end repeat
  return anims
end

on zeroAllRegPoints
  mlist = [:]
  p = point(0, 0)
  f = the frame
  beginRecording()
  repeat with i = 1 to the lastChannel
    s = sprite(i)
    m = s.member
    if m.number > 0 then
      if voidp(mlist.getaProp(m)) then
        mlist.setaProp(m, m.regPoint)
      end if
      rp = mlist.getaProp(m)
      if rp <> p then
        scale = point(float(s.width) / m.rect.width, float(s.height) / m.rect.height)
        s.loc = s.loc - (rp * scale)
        m.regPoint = p
      end if
    end if
  end repeat
  endRecording()
  go(f)
end

on convertToAlpha mem, bgcol
  if voidp(bgcol) then
    bgcol = rgb(255, 255, 255)
  end if
  img = mem.image.duplicate()
  img.useAlpha = 1
  cst = mem.castLibNum
  num = mem.number
  name = mem.name
  newmem = new(#bitmap, castLib(cst))
  newmem.name = name
  newmem.image = image(img.rect.width, img.rect.height, 32)
  newmem.image.setAlpha(image(img.rect.width, img.rect.height, 8))
  newmem.image.copyPixels(img, img.rect, img.rect, [#ink: 36, #bgColor: bgcol])
end

on getSitterRegPoint sid
  if voidp(glob) then
    return 
  end if
  return glob.managers.config.sitterRegPoints[sid]
end

on _makecolor alldata, colstr
  fr = alldata[#frames]
  if not voidp(fr) then
    repeat with dat in fr
      c = dat[#cast]
      if not voidp(c) then
        if c contains "gfx.people." then
          dat[#cast] = "gfx.people." & colstr
        end if
        m = member(dat.member, dat.cast)
        if m.number = -1 then
          complain("misnamed cast member (" && dat.member && dat.cast && ")")
        end if
      end if
    end repeat
  end if
  repeat with i in alldata.children
    _makecolor(i, colstr)
  end repeat
end

on makeColor str, colstr
  dat = getNodeData(str).duplicate()
  _makecolor(dat, colstr)
  memname = str & "_" & colstr & "_layout"
  m = member(memname, "scripts.layout")
  if m.number = -1 then
    m = new(#text, castLib("scripts.layout"))
    m.name = memname
  end if
  m.text = devoid(string(dat))
  dm = member(str & "_dummy", castLib("gfx.layout"))
  if voidp(dm) then
    dm = makeDummy(str)
  end if
  dm2 = makeDummy(str & "_" & colstr)
  dm2.regPoint = dm.regPoint
  setSitterRegPoint(str, dm.regPoint)
  return dat
end

on setSitterRegPoint id, rp
  m = member("sitter_regpoints", "scripts.layout")
  if m.number = -1 then
    m = new(#text, castLib("scripts.layout"))
    m.name = "sitter_regpoints"
  end if
  val = value(devoid(m.text))
  if not (val.ilk = #propList) then
    val = [:]
  end if
  val[id] = rp
  m.text = devoid(string(val))
end

on makeColors
  repeat with j in ["old_male", "business_female", "young_female", "boy", "man"]
    repeat with i in ["red", "blue", "green", "purple", "yellow"]
      makeColor(j, i)
    end repeat
  end repeat
  setSitterRegPoint("critic", member("critic_dummy", "gfx.layout").regPoint)
end

on cropChars
  repeat with clr in ["blue", "red", "green", "yellow", "purple", "none"]
    cst = "gfx.people." & clr
    repeat with i = 1 to the number of castMembers of castLib cst
      m = member(i, cst)
      mp1 = member(i + 1, cst)
      if (m.type = #empty) or (mp1.type = #empty) then
        next repeat
      end if
      if (m.paletteRef <> #systemWin) and (mp1.paletteRef = #systemWin) then
        cropWithAlpha(m, mp1)
      end if
    end repeat
  end repeat
end

on cropWithAlpha m, msk
  thresh = 128
  rp = m.regPoint
  img = image(m.width, m.height, 8)
  img.paletteRef = m.paletteRef
  img.fill(img.rect, color(255, 255, 255))
  r = m.rect
  img.copyPixels(m.image, r, r, [#maskImage: msk.image])
  m.image = img
  m.regPoint = rp
end

on putDots m, col
  if voidp(col) then
    col = rgb(0, 0, 0)
  end if
  m.image.setPixel(0, 0, col)
  m.image.setPixel(m.width - 1, m.height - 1, col)
end

on putDotsAll col
  if voidp(col) then
    col = rgb(0, 0, 0)
  end if
  repeat with clr in ["blue", "red", "green", "yellow", "purple", "none"]
    clibname = "gfx.people." & clr
    repeat with i = 1 to the number of castMembers of castLib clibname
      m = member(i, clibname)
      if m.type <> #bitmap then
        next repeat
      end if
      if m.paletteRef <> #systemWin then
        next repeat
      end if
      putDots(m, col)
    end repeat
  end repeat
end
