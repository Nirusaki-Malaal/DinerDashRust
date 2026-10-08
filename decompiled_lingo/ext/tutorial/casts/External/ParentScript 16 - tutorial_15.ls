property done, roomgroups
global glob

on startStep me
  done = 0
  roomgroups = glob.managers.game.room.sitterGroups
  if roomgroups.count then
    return 
  end if
  return 1
end

on checkDone me
  if done or voidp(roomgroups) then
    return 
  end if
  done = roomgroups.count = 2
  return done
end
