property mgr, name, node_mgr, game_mgr, gameGrp, titleGrp, stageSounds

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  gameGrp = node_mgr.getGroup("game")
  titleGrp = node_mgr.getGroup("titles")
  game_mgr = m.game_mgr
  stageSounds = ["sfx_r_diner", "sfx_r_tiki", "sfx_r_seafood", "sfx_r_chez", "sfx_r_goddess"]
  name = #play
  return me
end

on destroy me
  mgr = VOID
  node_mgr = VOID
end

on enterMode me, previousMode
  titleGrp.setVisible(0)
  game_mgr.setPaused(0)
  pm = previousMode.name
  if pm = #startLevel then
    game_mgr.startLevel()
  else
    if (pm = #quit) or (pm = #endless_shop) then
      nothing()
    else
      game_mgr.startGame()
      SndSFX(stageSounds[max(1, min(game_mgr.currentStage, 4))])
    end if
  end if
end

on event me, sym
end

on leaveMode me, nextMode
end
