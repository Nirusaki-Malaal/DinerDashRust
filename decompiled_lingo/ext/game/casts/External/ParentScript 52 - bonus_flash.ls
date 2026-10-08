property node, kids, showing, flashMS, numflashes, flashcount, flashtimer, visible

on new me, n
  node = n
  kids = node.children.duplicate()
  node.setZOffset(1000000)
  n.setVisible(0)
  visible = 0
  flashMS = 100
  numflashes = 10
  flashcount = 0
  flashtimer = script("timeout").new()
  flashtimer.setCallback([#target: me, #handler: #flash])
  flashtimer.setPeriod(flashMS)
  return me
end

on flash me
  flashcount = flashcount - visible
  visible = not visible
  node.setVisible(visible)
  if flashcount = 0 then
    flashtimer.stop()
  end if
end

on show me, str
  call(#setText, kids, string(str))
  flashcount = numflashes
  flashtimer.start()
  me.flash()
end

on hide me
  node.setVisible(0)
  flashtimer.stop()
end

on destroy me
  flashtimer.destroy()
  flashtimer = VOID
  node = VOID
  kids.deleteAll()
end
