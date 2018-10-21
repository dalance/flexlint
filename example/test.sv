module test ();

    wire clk;
    wire rst;

    reg reg1;
    always @ ( posedge clk ) begin
        reg1 <= 0;
    end

    reg reg2;
    always_ff @ ( posedge clk or negedge rst ) begin
        if ( rst )
            reg2 <= 0;
        else
            reg2 <= 1;
    end

    reg reg3;
    always_comb begin
        reg3 = reg0 | reg1;
    end

endmodule
