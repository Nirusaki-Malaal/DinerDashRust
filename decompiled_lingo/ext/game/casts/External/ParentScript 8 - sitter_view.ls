property sitter, node, facenode, legnode, home, state, facemems, facenum, faceAnimator, moodAnims, stateAnims, inFront, bodynode, alphaspeed, alphadir, alphacount, flashtimer, flashstate, flashcount, numflashes, flashtime, snapping, startLoc, diffLoc, snapDurMS, currentSnapMS, heldObject, room_mgr
global glob

on new me, Args
  node = Args.node
  legnode = Args.legnode
  bodynode = node["body"]
  heldObject = node["held_object"]
  heldObject.setVisible(0)
  alphadir = 1
  alphacount = 0
  alphaspeed = 2.5
  snapDurMS = 75.0
  currentSnapMS = 0.0
  room_mgr = glob.managers.game.room
  facenode = node["face"]
  facenode.setVisible(0)
  numflashes = 5
  flashtime = 200
  flashcount = 0
  flashstate = 1
  flashtimer = script("timeout").new()
  flashtimer.setCallback([#target: me, #handler: #flash])
  flashtimer.period = flashtime
  cb = VOID
  stateAnims = [#drinking: [#name: "drinking", #Args: [#loopCount: 1, #frameMS: 1000, #callback: [#target: me, #handler: #setState]]], #deciding: [#name: "deciding", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #ready: [#name: "ready", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #snack: [#name: "ready", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #ordered: [#name: "neutral", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #waiting: [#name: "neutral", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #prefood: [#name: "prefood", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #eating: [#name: "eating", #Args: [#loopCount: 0, #frameMS: 400, #callback: cb]], #check: [#name: "ready", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #mad: [#name: "mad", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #happy: [#name: "happy", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #neutral: [#name: "neutral", #Args: [#loopCount: 1, #frameMS: 2000, #callback: cb]], #leaving: [#name: "leaving", #Args: [#loopCount: 1, #frameMS: 750, #callback: [#target: me, #handler: #setState, #Args: #left]]], #left: [#name: "left", #Args: [#loopCount: 1, #frameMS: 750, #callback: cb]], #ditching: [#name: "ditching", #Args: [#loopCount: 1, #frameMS: 750, #callback: [#target: me, #handler: #setState, #Args: #ditched]]], #ditched: [#name: "ditched", #Args: [#loopCount: 1, #frameMS: 750, #callback: cb]]]
  moodAnims = ["mad", "annoyed", "dot", "happy"]
  dotmem = member("dot", "gfx.people")
  me.update()
  return me
end

on setFoodPickedUp me, b
  if b then
    me.setState(#prefood)
  else
    me.setState(sitter.state)
  end if
end

on setState me, m
  if voidp(legnode) then
    return 
  end if
  legnode.setVisible((m <> #left) and (m <> #ditched) and not voidp(sitter))
  if voidp(sitter) then
    return 
  end if
  if voidp(m) then
    m = sitter.state
  end if
  state = m
  if m = #scoring then
    md = sitter.mood
    if md > 7 then
      m = #happy
    else
      if md < 4 then
        m = #mad
      else
        m = #neutral
      end if
    end if
  end if
  anim = stateAnims[m]
  if not voidp(anim) then
    bodynode.animator.stop()
    bodynode.setAnimation(anim.name, anim.Args)
  end if
  if m = #drinking then
    heldObject.setAnimation("drink", [#animProps: [#loc]])
    heldObject.setMember(member("drink_cust", "gfx." & room_mgr.currentStage))
    heldObject.setVisible(1)
  else
    if m = #deciding then
      heldObject.setAnimation("menu", [#animProps: [#loc, #member]])
      heldObject.setVisible(1)
    else
      heldObject.setVisible(0)
    end if
  end if
  bodynode.setVisible(1)
end

on getMoodNum me, m
  return max(1, min(integer(m / 2.5), 4))
end

on destroy me
  if not voidp(sitter) then
    sitter.removeView(me)
  end if
  unsubscribe(me, #exitFrame)
  setting = VOID
  node = VOID
  bodynodes = VOID
  facenode.setVisible(0)
  facenode = VOID
  if not voidp(faceAnimator) then
    faceAnimator.destroy()
    faceAnimator = VOID
  end if
  legnode.setVisible(0)
  legnode = VOID
  flashtimer.destroy()
  flashtimer = VOID
end

on snapTo me, nd, inFrnt, xflip, instant
  if voidp(nd) then
    return 
  end if
  node.setXFlip(xflip)
  legnode.setXFlip(xflip)
  startLoc = node.getLoc()
  diffLoc = nd.getLoc() - startLoc
  if inFrnt then
    inFront = 1
  else
    inFront = 0
  end if
  currentSnapMS = 0
  snapping = 1
  if instant then
    me.stopSnapping()
  else
    subscribe(me, #exitFrame)
  end if
end

on exitFrame me, ms
  if snapping then
    currentSnapMS = currentSnapMS + ms
    if currentSnapMS > snapDurMS then
      me.stopSnapping()
      return 
    end if
    pt = startLoc + (diffLoc * (currentSnapMS / snapDurMS))
    node.setLoc(pt)
    legnode.setLoc(pt)
  end if
end

on stopSnapping me
  snapping = 0
  unsubscribe(me, #exitFrame)
  pt = startLoc + diffLoc
  node.setLoc(pt)
  legnode.setLoc(pt)
end

on setSitter me, s, old_view
  if sitter = s then
    return 
  end if
  if not voidp(sitter) then
    sitter.removeView(me)
    call(#stop, [facenode.animator, bodynode.animator])
  end if
  sitter = s
  if not voidp(sitter) then
    sitter.addView(me)
    col = sitter.color
    facenum = sitter.sitterGroup.mood
    g = sitter.sitterGroup.graphicType
    if g <> "critic" then
      node.setDummyID(g & "_" & col)
    else
      node.setDummyID(g)
    end if
    facenode = node["face"]
    bodynode = node["body"]
    heldObject = node["held_object"]
    rp = getSitterRegPoint(g)
    node.setRegPoint(rp)
    lm = member(g & "_legs", "gfx.people." & col)
    lm_r = member(lm.name, "gfx.people." & col)
    legnode.setMember(lm)
    legnode.setRegPoint(lm_r.regPoint)
    facenode.setAnimation("dot", [#loopCount: 1])
    me.setState(sitter.state)
    me.updateMood()
  end if
  me.update()
end

on drink me
  me.setState(#drinking)
end

on updateMood me
  if voidp(sitter) then
    return 
  end if
  num = sitter.sitterGroup.mood
  if num = 0 then
    return 
  end if
  if num <> facenum then
    if state = #drinking then
      cb = VOID
    else
      cb = [#target: me, #handler: #setState]
    end if
    facenode.setVisible(1)
    if num > facenum then
      facenode.setAnimation("better", [#loopCount: 1, #callback: cb])
      facenode.queueAnimation("dot", [#loopCount: 1])
      if not voidp(cb) then
        bodynode.setAnimation("happy")
      end if
    else
      if num > 1 then
        facenode.setAnimation("worse", [#loopCount: 1, #callback: cb])
        facenode.queueAnimation("dot", [#loopCount: 1])
        if not voidp(cb) then
          bodynode.setAnimation("mad")
        end if
      else
        facenode.setAnimation("worse", [#loopCount: 0])
        if not voidp(cb) then
          bodynode.setAnimation("mad")
        end if
      end if
    end if
    facenum = num
  end if
end

on update me
  if voidp(sitter) then
    call(#stop, [facenode.animator, bodynode.animator])
    node.setVisible(0)
    bodynode.setVisible(0)
    legnode.setVisible(0)
  else
    s = sitter.state
    me.updateMood()
    node.setVisible(1)
    bodynode.setVisible(1)
    legnode.setVisible(1)
  end if
end
