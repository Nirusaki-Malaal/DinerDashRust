property dragicon, done
global glob

on startStep me
  done = 0
  dragicon = glob.managers.game.queue.dragicon
  return not voidp(dragicon)
end

on checkDone me
  if done or voidp(dragicon) then
    return 
  end if
  done = dragicon.dragging
  return done
end
