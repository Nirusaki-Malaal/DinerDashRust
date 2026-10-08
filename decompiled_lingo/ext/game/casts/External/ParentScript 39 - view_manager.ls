property glob, node_mgr, views, sitterAnims, recycleList

on new me, g
  glob = g
  views = [:]
  views["6top"] = [#script: script("6top_view", "scripts.game"), #prebuild: 0, #views: [], #free: []]
  views["4top"] = [#script: script("4top_view", "scripts.game"), #prebuild: 0, #views: [], #free: []]
  views["2top"] = [#script: script("2top_view", "scripts.game"), #prebuild: 0, #views: [], #free: []]
  views["colorbonus"] = [#script: script("colorbonus", "scripts.game"), #prebuild: 1, #views: [], #free: []]
  views["multbonus"] = [#script: script("multbonus", "scripts.game"), #prebuild: 1, #views: [], #free: []]
  views["penalty"] = [#script: script("penalty", "scripts.game"), #prebuild: 1, #views: [], #free: []]
  views["tip"] = [#script: script("tip_bonus", "scripts.game"), #prebuild: 1, #views: [], #free: []]
  recycleList = ["colorbonus", "multbonus", "penalty", "tip"]
  return me
end

on showChairBonus me, nd, pts
  vu = me.getView("colorbonus")
  if voidp(vu) then
    return 
  end if
  vu.node.setParent(nd.parent)
  vu.node.setLoc(nd.getLoc())
  vu.show(pts)
end

on showTableBonus me, nd, arg
  if arg[#score] = 0 then
    return 
  end if
  vu = me.getView("multbonus")
  if voidp(vu) then
    return 
  end if
  vu.node.setLocRel(nd.getLocRel())
  vu.show(arg)
end

on showPenalty me, nd, arg
  vu = me.getView("penalty")
  if voidp(vu) then
    return 
  end if
  vu.node.setLocRel(nd.getLocRel())
  vu.show(arg)
end

on showTip me, nd, arg
  vu = me.getView("tip")
  if voidp(vu) then
    return 
  end if
  vu.node.setLocRel(nd.getLocRel())
  vu.show(arg)
end

on init me
  tp = []
  repeat with i in glob.managers.config.group.types
    tp.add(i.graphicType)
  end repeat
  sitterAnims = [:]
  repeat with t in tp
    repeat with c in ["red", "green", "blue", "purple", "yellow"]
      k = t & "_" & c
      n = getNodeData(k)
      if voidp(n) then
        next repeat
      end if
      sitterAnims[k] = [#body: getAnimData(n.children["body"].frames), #face: getAnimData(n.children["face"].frames)]
    end repeat
  end repeat
  node_mgr = glob.managers.node
  tempnode = node_mgr.ROOT.newNode(getNodeData("please_wait"))
  tempnode.setLoc(point(0, 0))
  tempnode.setZOffset(10000000)
  tempnode.setVisible(1)
  repeat with ls in views
    repeat with j = 1 to ls.prebuild
      vu = ls.script.new()
      ls.views.add(vu)
      ls.free.add(vu)
    end repeat
  end repeat
  tempnode.destroy()
  tempnode = VOID
end

on destroy me
  node_mgr = VOID
  glob = VOID
  repeat with ls in views
    ls.free.deleteAll()
    call(#destroy, ls.views)
    ls.views.deleteAll()
  end repeat
end

on getView me, sym
  ls = views[sym]
  if voidp(ls) then
    complain("no view type '" & sym & "'")
    return 
  end if
  fr = ls.free
  if fr.count then
    vu = fr[1]
    fr.deleteAt(1)
    return vu
  end if
  vu = callAndPutMS(#new, ls.script)
  ls.views.add(vu)
  return vu
end

on returnView me, vu
  if vu.type = "4top" then
    nothing()
  end if
  ls = views[vu.type]
  if voidp(ls) then
    return 
  end if
  if ls.free.getOne(vu) <> 0 then
    complain("tried to return view already in free list")
    return 
  end if
  if ls.views.getOne(vu) = 0 then
    complain("tried to return view not created by this view mgr")
    return 
  end if
  vu.setObject(VOID)
  if recycleList.getOne(vu.type) and (ls.free.count < ls.prebuild) then
    ls.free.add(vu)
  else
    ls.views.deleteOne(vu)
    vu.destroy()
  end if
end
