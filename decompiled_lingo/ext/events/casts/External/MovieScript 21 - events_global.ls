global glob, time_mgr, debug

on prepareMovie
  set the exitLock to 1
  the debugPlaybackEnabled = 0
  go(1)
  _movie.displayTemplate.resizable = 0
  _movie.displayTemplate.titlebarOptions.minimizebox = 1
  if the runMode contains "Projector" then
    helpDirectorFindCastlibs()
  end if
  puppetTempo(60)
  put "setting up res xtra"
  setupResXtra()
  glob = script("glob").new()
  updateStage()
end

on startMovie
  if the runMode = "Author" then
    updateVersionText()
  end if
  time_mgr = glob.managers.time
  go(marker("fs_pause"))
end

on closeRequest
  if glob.voidp then
    halt()
  end if
  glob.closeRequested = 1
end

on mouseDown
  if glob.voidp then
    return 
  end if
  glob.mouseDown()
end

on mouseUp
  if glob.voidp then
    return 
  end if
  glob.mouseUp()
end

on keyDown
  if glob.voidp then
    return 
  end if
  glob.keyDown(the key)
end

on exitFrame
  if glob.voidp then
    return 
  end if
  glob.exitFrame()
end

on stopMovie
  if not (the runMode contains "Author") then
    return 
  end if
  if voidp(glob) then
    return 
  end if
  glob.destroy()
  glob = VOID
  repeat with i = 1 to the number of castMembers of castLib "dynamic"
    erase(member(i, "dynamic"))
  end repeat
end

on complain str
  if debug then
    alert(str)
  end if
end
