on fadeNodeTo n, val, ms, cb
  return script("node_tween", "scripts.anim").new([#node: n, #start: n.getAlpha(), #end: val, #ms: ms, #setter: #setAlpha, #callback: cb])
end

on flashNode Args
  n = Args[#node]
  times = Args[#loopCount]
  ms = Args[#ms]
  inv = Args[#invert]
  grp = Args[#group]
  cb = Args[#callback]
  ad = []
  if voidp(ms) then
    ms = 500
  end if
  if voidp(times) then
    times = 3
  end if
  if voidp(inv) then
    inv = 0
  end if
  repeat with i = 1 to times
    ad.add([#visible: inv])
    ad.add([#visible: not inv])
  end repeat
  anim = ["flash": ad]
  obj = script("node_anim").new([#node: n, #data: anim, #group: grp])
  obj.setAnimData(anim)
  obj.setAnimation("flash", [#loopCount: 1, #frameMS: ms, #callback: cb, #animProps: [#visible]])
  return obj
end

on moveNodeTo n, pt, ms, cb
  if n.ilk = #propList then
    pt = n.pt
    ms = n[#ms]
    cb = n[#cb]
    n = n.node
  end if
  script("node_tween", "scripts.anim").new([#node: n, #start: n.getLoc(), #end: pt, #ms: ms, #setter: #setLoc, #callback: cb])
end
