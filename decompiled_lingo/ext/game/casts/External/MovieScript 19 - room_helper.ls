global glob

on testRoomMgr
  s = glob.managers.game.room_manager
  s.loadLevel()
  t1 = s.tables[1]
  t2 = s.tables[2]
  g1 = s.waitingGroups[1]
  g2 = s.waitingGroups[2]
  t1.seatGroup(g1, t1.chairs[1])
  t2.seatGroup(g2, t2.chairs[2])
  gr = s.grid.grid[3]
  repeat with i in gr
    put i.occupant.type
  end repeat
  put "asdf"
  g2.leaveTable()
  put "asdf"
  s.destroy()
  s = VOID
end

on putgridoccupants me
  glob.managers.game.room.grid.putOccupants()
end
