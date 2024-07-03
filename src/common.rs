

// 词法记号标签
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tag {
    ERR,                                 // 错误，异常
    END,                                 // 文件结束标记
    ID,                                  // 标识符
    KwInt,
    KwChar,
    KwVoid,             // 数据类型
    KwExtern,                           // extern
    NUM, CH, STR,                        // 字面量
    NOT, LEA,
    ADD, SUB, MUL, DIV, MOD,             // 单目运算符!-&*
    INC, DEC,
    GT, GE, LT, LE, EQU, NEQU,           // 比较运算符
    AND, OR,                             // 逻辑运算符
    LPAREN, RPAREN,                      // ()
    LBRACK, RBRACK,						 // []
    LBRACE, RBRACE,						 // {}
    COMMA, COLON, SEMICON,				 // 逗号,冒号,分号
    ASSIGN,								 // 赋值
    KwIf,
    KwElse,						 // if-else
    KwSwitch,
    KwCase,
    KwDefault,		 // swicth-case-deault
    KwWhile,
    KwDo,
    KwFor,				 // 循环
    KwBreak,
    KwContinue,
    KwReturn         // break, continue, return
}

pub enum LexError {
    StrNoRQution,		    //字符串没有右引号
    NumBinType,				//2进制数没有实体数据
    NumHexType,				//16进制数没有实体数据
    CharNoRQution,		    //字符没有右引号
    CharNoData,				//字符没有数据
    OrNoPair,					//||只有一个|
    CommentNoEnd,			    //多行注释没有正常结束
    TokenNoExist                //不存在的词法记号
}

// 语法错误码
pub enum SynError {
    TypeLost,					//类型
    TypeWrong,
    IdLost,						//标志符
    IdWrong,
    NumLost,						//数组长度
    NumWrong,
    LiteralLost,				//常量
    LiteralWrong,
    CommaLost,					//逗号
    CommaWrong,
    SemiconLost,				//分号
    SemiconWrong,
    AssignLost,				//=
    AssignWrong,
    ColonLost,					//冒号
    ColonWrong,
    WhileLost,					//while
    WhileWrong,
    LparenLost,				//(
    LparenWrong,
    RparenLost,				//)
    RparenWrong,
    LbrackLost,				//[
    LbrackWrong,
    RbrackLost,				//]
    RbrackWrong,
    LbraceLost,				//{
    LbraceWrong,
    RbraceLost,				//}
    RbraceWrong
}

// 语义错误码
pub enum SemError {
    VarReDef,					//变量重定义
    FunReDef,					//函数重定义
    VarUnDec,					//变量未声明
    FunUnDec,					//函数未声明
    FunDecErr,				//函数声明与定义不匹配
    FunCallErr,				//行参实参不匹配
    DecInitDeny,			//声明不允许初始化
    ExternFunDef,			//函数声明不能使用extern
    ArrayLenInvalid,	//数组长度无效
    VarInitErr,				//变量初始化类型错误
    GlbInitErr,				//全局变量初始化值不是常量
    VoidVar,						//void变量
    ExprNotLeftVal,	//无效的左值表达式
    AssignTypeErr,		//赋值类型不匹配
    ExprIsBase,				//表达式不能是基本类型
    ExprNotBase,			//表达式不是基本类型
    ArrTypeErr,				//数组运算类型错误
    ExprIsVoid,				//表达式不能是VOID类型
    BreakErr,					//break不在循环或switch-case中
    ContinueErr,				//continue不在循环中
    ReturnErr                    //return语句和函数返回值类型不匹配
}
