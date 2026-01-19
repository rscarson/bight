function REL(deltaX, deltaY)
  return GET(POS.x + deltaX, POS.y + deltaY)
end

function RELX(deltaX)
  return REL(0, deltaX)
end

function RELY(deltaY)
  return REL(0, deltaY)
end

setmetatable(_G, {
  __index = function(_, index)
    if (index == "POS") then return THIS_POS() end
    return CELL_POS(index)
  end
})
