function update()
    local actual_title = app:get_title()
    -- local actual_fps = app:get_fps() - 1;
    if(actual_fps> 0) then
        app:set_fps(actual_fps)
    end
    app:set_title("Herrumbre - FPS: ".. app:get_fps())
end
