property done, flo, sitterGroups
global glob

on startStep me
  done = 0
  sitterGroups = glob.managers.game.room.sitterGroups
  if sitterGroups.count <> 2 then
    return 
  end if
  flo = glob.managers.game.room.waiter
  return 1
end

on checkDone me
  if done then
    return 
  end if
  gottenFood = 1
  repeat with s in sitterGroups
    nm = s.state.name
    gottenFood = gottenFood and ((nm = #eating) or (nm = #check))
  end repeat
  done = not flo.hasPayloadType(#food) and gottenFood
  return done
end
